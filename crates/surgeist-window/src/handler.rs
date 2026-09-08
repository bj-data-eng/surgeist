use super::{Close, Closed, Context, Event, Frame, Input, Ready, Resize, Result};

/// Consumer callbacks delivered by the shared native and deterministic lifecycle pump.
///
/// Each callback receives transaction-local context or scope. Its queued commands
/// and actions commit only when it returns `Ok`; normalization, planning,
/// capability checks, and backend application then remain able to fail. A
/// callback error, committed-command failure, or terminal action retains the
/// first terminal outcome and suppresses later callbacks. Clipboard calls made
/// through a callback context are already-completed external I/O and are not
/// rolled back. Every default callback is a no-op.
pub trait Handler {
    /// Runs after per-window resume events and after startup validation succeeds.
    ///
    /// On later resumes, each live window first reaches [`Handler::event`] with a
    /// `Resumed` event in deterministic id order. The default does nothing.
    fn resume(&mut self, _cx: &mut Context<'_>) -> Result<()> {
        Ok(())
    }

    /// Runs after per-window suspend events in deterministic id order.
    ///
    /// It is not called before the first successful resume. The default does nothing.
    fn suspend(&mut self, _cx: &mut Context<'_>) -> Result<()> {
        Ok(())
    }

    /// Runs once after a successful `Created` event for a still-live window.
    ///
    /// A successful ready callback schedules a next-frame draw by default; exit,
    /// close, or failure during `Created` suppresses ready delivery. The default
    /// does nothing.
    fn ready(&mut self, _win: &mut Ready<'_>) -> Result<()> {
        Ok(())
    }

    /// Receives observed resize or scale-factor changes after their snapshot update.
    ///
    /// A successful resize callback schedules a next-frame draw by default. The
    /// default does nothing.
    fn resize(&mut self, _win: &mut Resize<'_>) -> Result<()> {
        Ok(())
    }

    /// Receives normalized input for its live target window.
    ///
    /// The pump routes [`crate::EventKind::Input`] here, rather than through
    /// [`Handler::event`], after native input lowering. Input itself does not
    /// mutate the snapshot; the scope exposes the target's committed state at
    /// dispatch. The default does nothing.
    fn input(&mut self, _input: &mut Input<'_>) -> Result<()> {
        Ok(())
    }

    /// Receives remaining normalized host events after their observed transition.
    ///
    /// The pump routes every retained [`crate::EventKind`] except input and
    /// metric changes here: input uses [`Handler::input`] and resize or scale
    /// changes use [`Handler::resize`]. Producer transitions such as focus,
    /// position, occlusion, theme, and creation have
    /// already committed their snapshots before delivery. `Created` arrives here
    /// before [`Handler::ready`]; native close and final destruction instead use
    /// [`Handler::close`] and [`Handler::closed`]. The default does nothing.
    fn event(&mut self, _event: &mut Event<'_>) -> Result<()> {
        Ok(())
    }

    /// Receives a close request and decides whether to begin closing the window.
    ///
    /// The default accepts the request. Calling [`Close::cancel`](super::Close::cancel)
    /// leaves the window live; accepted close begins shared teardown and eventually
    /// delivers [`Handler::closed`] exactly once when ownership ends.
    fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
        close.close();
        Ok(())
    }

    /// Receives the final snapshot once after closing has completed.
    ///
    /// The window is no longer live, and this callback can only queue global work
    /// such as exit. The default does nothing.
    fn closed(&mut self, _closed: &mut Closed<'_>) -> Result<()> {
        Ok(())
    }

    /// Receives one scheduled draw for a live window.
    ///
    /// Native redraw delivery and deterministic runner draw dispatch both use
    /// this route. The scope observes committed live state; its `draw`, `again`,
    /// and `at` methods schedule later work only after successful completion.
    /// The default does nothing.
    fn draw(&mut self, _frame: &mut Frame<'_>) -> Result<()> {
        Ok(())
    }

    /// Returns whether [`Handler::idle`] should run before the event loop waits.
    ///
    /// The pump reads this on each idle opportunity after first resume. The default
    /// is `false`.
    fn wants_idle(&self) -> bool {
        false
    }

    /// Runs before the event loop waits when [`Handler::wants_idle`] is `true`.
    ///
    /// It is skipped before first resume and uses the same callback transaction as
    /// lifecycle callbacks. The default does nothing.
    fn idle(&mut self, _cx: &mut Context<'_>) -> Result<()> {
        Ok(())
    }
}
