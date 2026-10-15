//! The obsolete but normative marquee turned-on slot.

/// Marquee's actual turned-on state, independent of layout or animation.
///
/// HTML 877d6146487f [#the-marquee-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-marquee-element)
/// creates marquees turned on. The start and stop methods set this slot; they
/// do not themselves establish renderer, timing, execution or event capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlMarqueeState {
    turned_on: bool,
}

impl HtmlMarqueeState {
    pub const fn fresh() -> Self {
        Self { turned_on: true }
    }

    pub const fn turned_on(self) -> bool {
        self.turned_on
    }

    pub fn start(&mut self) {
        self.turned_on = true;
    }

    pub fn stop(&mut self) {
        self.turned_on = false;
    }
}
