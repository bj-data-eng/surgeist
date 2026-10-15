//! The obsolete but normative native marquee state.

/// Marquee's actual native state, independent of layout or animation.
///
/// HTML 877d6146487f [#the-marquee-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-marquee-element)
/// creates marquees turned on with current loop index zero. Start and stop
/// change only the turned-on slot, preserving the current loop index.
///
/// The supported native domain has no rendering-layer increment operation, so
/// its index remains actually zero. This is not an unavailable observation or
/// a general counter with a machine-integer magnitude bound. Installing a
/// rendering increment capability must extend this closed domain first.
/// Copying this value retains actual state; fresh native creation is separate.
/// No renderer, timing, execution or event capability is established here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlMarqueeState {
    turned_on: bool,
    current_loop_index: NativeLoopIndex,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NativeLoopIndex {
    Zero,
}

impl HtmlMarqueeState {
    pub const fn fresh() -> Self {
        Self {
            turned_on: true,
            current_loop_index: NativeLoopIndex::Zero,
        }
    }

    pub const fn turned_on(self) -> bool {
        self.turned_on
    }

    /// Actual current loop index in the supported native zero-index domain.
    pub const fn current_loop_index(self) -> u128 {
        match self.current_loop_index {
            NativeLoopIndex::Zero => 0,
        }
    }

    pub fn start(&mut self) {
        self.turned_on = true;
    }

    pub fn stop(&mut self) {
        self.turned_on = false;
    }
}
