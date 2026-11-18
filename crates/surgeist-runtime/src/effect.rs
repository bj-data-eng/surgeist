//! Effects emitted by reducers and the record of how the runtime handled them.
//!
//! An [`AppEffect`] is an app-authored effect with a closed runtime payload ([`AppEffectPayload`]); the effects of one
//! successful reducer commit form an ordered [`EffectBatch`]. The runtime records how it handled each effect as an
//! [`EffectOutcome`] with an [`EffectDisposition`]. Work that belongs to the host adapter, such as the service, task,
//! resource and persistence requests, is forwarded unchanged as a [`RuntimeIntent`].

pub use crate::internal::effect::{
    AppEffect, AppEffectPayload, CallServiceEffect, CancelTaskEffect, DiagnosticEffect,
    EffectBatch, EffectDisposition, EffectKindId, EffectOutcome, InvalidateResourceEffect,
    LoadResourceEffect, PersistEffect, RedrawTarget, ReprioritizeTaskEffect, RequestRedrawEffect,
    RuntimeIntent, ServiceDiagnosticEffect, StartServiceEffect, StartTaskEffect, StopServiceEffect,
};
