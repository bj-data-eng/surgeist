use std::mem;

use crate::{Error, PropertyKey, Value, ValueKind, WellKnownSymbol};

use super::composition::sealed;
use super::{Conversion, Converted, Descriptor, Failure, Nullable, Rule, Work};

/// A synchronous IDL sequence of a closed typed descriptor. Each element keeps
/// its own conversion custody; the outer result also owns the requested slots.
pub struct Sequence<D>(pub(super) D);

impl<D: Descriptor> Sequence<D> {
    pub fn sequence(self) -> Sequence<Self> {
        Sequence(self)
    }
    pub fn nullable(self) -> Nullable<Self> {
        Nullable(self)
    }
}
impl<D: Descriptor> Descriptor for Sequence<D> {
    type Output = Vec<Converted<D::Output>>;
}
impl<D: Descriptor> sealed::Convert<Vec<Converted<D::Output>>> for Sequence<D> {
    fn convert<'s>(
        &self,
        cx: &mut Conversion<'_, 's>,
        input: &Value<'s>,
    ) -> Result<Converted<Vec<Converted<D::Output>>>, Failure> {
        cx.work(Work::Value, 1).map_err(Failure)?;
        cx.sequence(&self.0, input)
    }
}

impl<'s> Conversion<'_, 's> {
    fn iterator_get(
        &mut self,
        object: &Value<'s>,
        key: PropertyKey<'_, 's>,
    ) -> Result<Value<'s>, Failure> {
        self.work(Work::IteratorOperation, 1).map_err(Failure)?;
        let result = self.scope.get(object, key);
        self.core(result)
    }

    fn iterator_call(
        &mut self,
        function: &Value<'s>,
        receiver: &Value<'s>,
    ) -> Result<Value<'s>, Failure> {
        self.work(Work::IteratorOperation, 1).map_err(Failure)?;
        let result = self.scope.is_callable(function);
        if !self.core(result)? {
            return Err(self.validation(Rule::Sequence));
        }
        let result = self.scope.call(function, receiver, &[]);
        self.core(result)
    }

    fn iterator_object(&mut self, value: &Value<'s>) -> Result<(), Failure> {
        let result = self.scope.kind(value);
        if self.core(result)? != ValueKind::Object {
            return Err(self.validation(Rule::Sequence));
        }
        Ok(())
    }

    fn sequence<D: Descriptor>(
        &mut self,
        ty: &D,
        input: &Value<'s>,
    ) -> Result<Converted<Vec<Converted<D::Output>>>, Failure> {
        self.iterator_object(input)?;
        let result = self.scope.well_known_symbol(WellKnownSymbol::Iterator);
        let symbol = self.core(result)?;
        // GetMethod and GetIteratorFromMethod cache these original values.
        let method = self.iterator_get(input, PropertyKey::Value(&symbol))?;
        let iterator = self.iterator_call(&method, input)?;
        self.iterator_object(&iterator)?;
        let next = self.iterator_get(&iterator, PropertyKey::String(&[110, 101, 120, 116]))?;
        let mut output = self.plain(Vec::new());
        let mut capacity = 0;
        loop {
            let result = self.iterator_call(&next, &iterator)?;
            self.iterator_object(&result)?;
            let done = self.iterator_get(&result, PropertyKey::String(&[100, 111, 110, 101]))?;
            let boolean = self.scope.boolean(&done);
            if self.core(boolean)? {
                return Ok(output);
            }
            self.work(Work::SequenceItem, 1).map_err(Failure)?;
            let value =
                self.iterator_get(&result, PropertyKey::String(&[118, 97, 108, 117, 101]))?;
            let item = self.value(ty, &value)?;
            if output.value.len() == capacity {
                self.grow_sequence(&mut output, &mut capacity)?;
            }
            output.value.push(item);
            // Pinned Web IDL has no IteratorClose on abrupt conversion. In
            // particular, neither this loop nor Rust cleanup reads `return`.
        }
    }

    fn grow_sequence<T>(
        &mut self,
        output: &mut Converted<Vec<Converted<T>>>,
        capacity: &mut usize,
    ) -> Result<(), Failure> {
        let next = if *capacity == 0 {
            4
        } else {
            capacity
                .checked_mul(2)
                .ok_or_else(|| self.host(Error::Allocation))?
        };
        let bytes = next
            .checked_mul(mem::size_of::<Converted<T>>())
            .filter(|bytes| *bytes <= isize::MAX as usize)
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| self.host(Error::Allocation))?;
        // Storage precedes its Vec local, so failure/unwind drops backing first.
        // Charge full new backing while the old backing remains live.
        let storage = self.reserve(bytes).map_err(Failure)?;
        let mut grown = Vec::new();
        grown
            .try_reserve_exact(next)
            .map_err(|_| self.host(Error::Allocation))?;
        let moves = u64::try_from(output.value.len()).map_err(|_| self.host(Error::Allocation))?;
        self.work(Work::SequenceMove, moves).map_err(Failure)?;
        for item in mem::take(&mut output.value) {
            grown.push(item);
        }
        // The consumed IntoIter freed old backing; only now release its charge.
        output.value = grown;
        drop(mem::replace(&mut output.storage, storage));
        *capacity = next;
        self.scope.check_active().map_err(Failure)
    }
}
