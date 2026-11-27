//! Native Object representation and attribute-triggered reconsideration.

/// The admitted representation phases before acquiring an external resource.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlObjectRepresentation {
    /// The original representation task has yet to select content.
    Undetermined,
    /// The representation task selected the object's children.
    FallbackContent,
}

/// Which higher-priority Object resource attributes are currently present.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlObjectResourcePrecedence {
    /// The legacy classid attribute is present, regardless of its value.
    ClassId,
    /// Data is present and classid is absent.
    Data,
    /// Both classid and data are absent.
    Neither,
}

/// A null-namespace Object resource attribute that was set, changed or removed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlObjectResourceAttribute {
    /// Legacy classid.
    ClassId,
    /// The resource URL attribute.
    Data,
    /// The declared content type.
    Type,
}

impl HtmlObjectResourceAttribute {
    /// Whether the mutation queues a new representation task, given current
    /// attribute presence. Each qualifying change queues its own occurrence.
    pub const fn queues_representation(self, precedence: HtmlObjectResourcePrecedence) -> bool {
        match self {
            Self::ClassId => true,
            Self::Data => !matches!(precedence, HtmlObjectResourcePrecedence::ClassId),
            Self::Type => matches!(precedence, HtmlObjectResourcePrecedence::Neither),
        }
    }
}
