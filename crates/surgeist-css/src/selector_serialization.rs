//! Authored selector emission. Symbolic nesting and scope anchors stay symbolic.
//! This provider owns selector grammar; enclosing rule formatting has a separate owner.
use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::*;
type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;
enum Event<'a> {
    Selector(&'a CssSelector),
    Compound(&'a CssCompoundSelector),
    CompoundTail(&'a CssCompoundSelector, u8, usize),
    ComplexParts(&'a [CssComplexSelectorPart], usize),
    Pseudo(&'a CssPseudoClass),
    Element(&'a CssPseudoElement),
    Attribute(&'a CssAttributeSelector),
    List(&'a [CssSelector], usize),
    RelativeList(&'a [CssRelativeSelector], usize),
    Text(&'a str),
    Nth(CssNthPattern),
}
fn push<'a>(work: &mut Vec<Event<'a>>, event: Event<'a>) -> Result<()> {
    work.try_reserve(1).map_err(|_| {
        CssSpecifiedValueSerializationError::new(
            CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
        )
    })?;
    work.push(event);
    Ok(())
}
impl CssSelector {
    /// Emits an authored selector, without resolving its symbolic ancestry.
    pub fn to_specified_css(&self) -> Result<String> {
        self.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Emits atomically with cumulative semantic-node and UTF-8 byte limits.
    pub fn to_specified_css_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.selector(self)?;
        Ok(writer.css)
    }
}
impl SpecifiedRuleWriter {
    pub(crate) fn node(&mut self) -> Result<()> {
        self.context.charge_input(1)?;
        self.context.charge_projection(1)
    }
    pub(crate) fn keyword(&mut self, text: &str) -> Result<()> {
        self.node()?;
        self.append(text)
    }
    pub(crate) fn combinator(&mut self, value: CssSelectorCombinator, leading: bool) -> Result<()> {
        self.append(match (value, leading) {
            (CssSelectorCombinator::Descendant, true) => "",
            (CssSelectorCombinator::Descendant, false) => " ",
            (CssSelectorCombinator::Child, true) => "> ",
            (CssSelectorCombinator::Child, false) => " > ",
            (CssSelectorCombinator::NextSibling, true) => "+ ",
            (CssSelectorCombinator::NextSibling, false) => " + ",
            (CssSelectorCombinator::SubsequentSibling, true) => "~ ",
            (CssSelectorCombinator::SubsequentSibling, false) => " ~ ",
        })
    }

    fn selector_identifier(&mut self, value: &str) -> Result<()> {
        if value.is_empty() || value.contains('\0') {
            return Err(CssSpecifiedValueSerializationError::new(
                CssSpecifiedValueSerializationErrorKind::UnrepresentableValue,
            ));
        }
        self.append_identifier(value)
    }

    fn name_prefix(&mut self, prefix: &CssQualifiedNamePrefix) -> Result<()> {
        match prefix {
            CssQualifiedNamePrefix::Unqualified => Ok(()),
            CssQualifiedNamePrefix::ExplicitNone => self.append("|"),
            CssQualifiedNamePrefix::Any => self.append("*|"),
            CssQualifiedNamePrefix::Named(name) => {
                self.selector_identifier(name.as_str())?;
                self.append("|")
            }
        }
    }
    pub(crate) fn selector(&mut self, selector: &CssSelector) -> Result<()> {
        let mut work = Vec::new();
        push(&mut work, Event::Selector(selector))?;
        while let Some(event) = work.pop() {
            match event {
                Event::Text(value) => self.append(value)?,
                Event::Selector(value) => match value {
                    CssSelector::Tag(value) => {
                        self.node()?;
                        self.selector_identifier(value)?;
                    }
                    CssSelector::Key(value) => {
                        self.node()?;
                        self.append("#")?;
                        self.selector_identifier(value)?;
                    }
                    CssSelector::Class(value) => {
                        self.node()?;
                        self.append(".")?;
                        self.selector_identifier(value)?;
                    }
                    CssSelector::PseudoClass(value) => push(&mut work, Event::Pseudo(value))?,
                    CssSelector::Compound(value) => push(&mut work, Event::Compound(value))?,
                    CssSelector::Complex(value) => {
                        self.node()?;
                        push(&mut work, Event::ComplexParts(value.rest(), 0))?;
                        push(&mut work, Event::Compound(value.first()))?;
                    }
                },
                Event::Compound(value) => {
                    self.node()?;
                    if let Some(name) = value.type_selector() {
                        self.node()?;
                        self.name_prefix(name.prefix())?;
                        if let Some(local) = name.local_name() {
                            self.selector_identifier(local)?;
                        } else {
                            self.append("*")?;
                        }
                    }
                    for _ in 0..value.nesting_selectors() {
                        self.keyword("&")?;
                    }
                    for _ in 0..value.scope_anchors() {
                        self.keyword("&")?;
                    }
                    for id in value.ids() {
                        self.node()?;
                        self.append("#")?;
                        self.selector_identifier(id)?;
                    }
                    for class in value.classes() {
                        self.node()?;
                        self.append(".")?;
                        self.selector_identifier(class)?;
                    }
                    push(&mut work, Event::CompoundTail(value, 0, 0))?;
                }
                Event::ComplexParts(parts, index) => {
                    if let Some(part) = parts.get(index) {
                        self.combinator(part.combinator(), false)?;
                        push(&mut work, Event::ComplexParts(parts, index + 1))?;
                        push(&mut work, Event::Compound(part.selector()))?;
                    }
                }
                Event::CompoundTail(value, phase, index) => {
                    let event = match phase {
                        0 => value.attributes().get(index).map(Event::Attribute),
                        1 => value.pseudo_classes().get(index).map(Event::Pseudo),
                        2 => value
                            .pseudo_elements()
                            .and_then(|s| s.segments().get(index))
                            .map(|s| match s {
                                CssPseudoElementSegment::PseudoElement(e) => Event::Element(e),
                                CssPseudoElementSegment::PseudoClass(p) => Event::Pseudo(p),
                            }),
                        _ => None,
                    };
                    if let Some(event) = event {
                        push(&mut work, Event::CompoundTail(value, phase, index + 1))?;
                        push(&mut work, event)?;
                    } else if phase < 2 {
                        push(&mut work, Event::CompoundTail(value, phase + 1, 0))?;
                    }
                }

                Event::List(values, index) => {
                    if let Some(value) = values.get(index) {
                        if index != 0 {
                            self.append(", ")?;
                        }
                        push(&mut work, Event::List(values, index + 1))?;
                        push(&mut work, Event::Selector(value))?;
                    }
                }
                Event::RelativeList(values, index) => {
                    if let Some(value) = values.get(index) {
                        if index != 0 {
                            self.append(", ")?;
                        }
                        self.node()?;
                        self.combinator(value.combinator(), true)?;
                        push(&mut work, Event::RelativeList(values, index + 1))?;
                        push(&mut work, Event::Selector(value.selector()))?;
                    }
                }
                Event::Attribute(value) => {
                    self.node()?;
                    self.append("[")?;
                    self.name_prefix(value.qualified_name().prefix())?;
                    self.selector_identifier(value.name().as_str())?;
                    let (operator, string) = match value.matcher() {
                        CssAttributeMatcher::Exists => ("", None),
                        CssAttributeMatcher::Equals(s) => ("=", Some(s)),
                        CssAttributeMatcher::Includes(s) => ("~=", Some(s)),
                        CssAttributeMatcher::DashMatch(s) => ("|=", Some(s)),
                        CssAttributeMatcher::Prefix(s) => ("^=", Some(s)),
                        CssAttributeMatcher::Suffix(s) => ("$=", Some(s)),
                        CssAttributeMatcher::Substring(s) => ("*=", Some(s)),
                    };
                    self.append(operator)?;
                    if let Some(string) = string {
                        self.append_string(string)?;
                    }
                    self.append(match value.case_sensitivity() {
                        CssAttributeCaseSensitivity::DocumentDefault => "",
                        CssAttributeCaseSensitivity::AsciiCaseInsensitive => " i",
                        CssAttributeCaseSensitivity::ExplicitSensitive => " s",
                    })?;
                    self.append("]")?;
                }
                Event::Nth(value) => {
                    self.node()?;
                    match value {
                        CssNthPattern::Odd => self.append("2n+1")?,
                        CssNthPattern::Even => self.append("2n")?,
                        CssNthPattern::Integer(value) => self.append(&value.to_string())?,
                        CssNthPattern::AnPlusB(value) => {
                            let (a, b) = (value.a(), value.b());
                            if a == 0 {
                                self.append(&b.to_string())?;
                            } else {
                                if a == -1 {
                                    self.append("-")?;
                                } else if a != 1 {
                                    self.append(&a.to_string())?;
                                }
                                self.append("n")?;
                                if b != 0 {
                                    self.append(if b < 0 { "-" } else { "+" })?;
                                    self.append(&i64::from(b).abs().to_string())?;
                                }
                            }
                        }
                    }
                }
                Event::Pseudo(value) => {
                    self.node()?;
                    match value {
                        CssPseudoClass::HostFunction(arg) | CssPseudoClass::HostContext(arg) => {
                            self.append(if matches!(value, CssPseudoClass::HostFunction(_)) {
                                ":host("
                            } else {
                                ":host-context("
                            })?;
                            push(&mut work, Event::Text(")"))?;
                            push(&mut work, Event::Compound(arg.compound()))?;
                        }
                        CssPseudoClass::Not(list)
                        | CssPseudoClass::Is(list)
                        | CssPseudoClass::Where(list) => {
                            self.append(match value {
                                CssPseudoClass::Not(_) => ":not(",
                                CssPseudoClass::Is(_) => ":is(",
                                _ => ":where(",
                            })?;
                            push(&mut work, Event::Text(")"))?;
                            push(&mut work, Event::List(list.selectors(), 0))?;
                        }
                        CssPseudoClass::Has(list) => {
                            self.append(":has(")?;
                            push(&mut work, Event::Text(")"))?;
                            push(&mut work, Event::RelativeList(list.selectors(), 0))?;
                        }
                        CssPseudoClass::NthChild(pattern)
                        | CssPseudoClass::NthLastChild(pattern) => {
                            self.append(if matches!(value, CssPseudoClass::NthChild(_)) {
                                ":nth-child("
                            } else {
                                ":nth-last-child("
                            })?;
                            push(&mut work, Event::Text(")"))?;
                            if let Some(list) = pattern.selector_list() {
                                push(&mut work, Event::List(list.selectors(), 0))?;
                                push(&mut work, Event::Text(" of "))?;
                            }
                            push(&mut work, Event::Nth(pattern.pattern()))?;
                        }
                        CssPseudoClass::NthOfType(pattern)
                        | CssPseudoClass::NthLastOfType(pattern) => {
                            self.append(if matches!(value, CssPseudoClass::NthOfType(_)) {
                                ":nth-of-type("
                            } else {
                                ":nth-last-of-type("
                            })?;
                            push(&mut work, Event::Text(")"))?;
                            push(&mut work, Event::Nth(*pattern))?;
                        }
                        CssPseudoClass::Dir(dir) => {
                            self.append(":dir(")?;
                            self.selector_identifier(dir.as_str())?;
                            self.append(")")?;
                        }
                        CssPseudoClass::Lang(list) => {
                            self.append(":lang(")?;
                            for (i, range) in list.ranges().iter().enumerate() {
                                if i != 0 {
                                    self.append(", ")?;
                                }
                                self.node()?;
                                self.append_string(range.as_str())?;
                            }
                            self.append(")")?;
                        }
                        CssPseudoClass::Host => self.append(":host")?,
                        CssPseudoClass::Root => self.append(":root")?,
                        CssPseudoClass::Scope => self.append(":scope")?,
                        CssPseudoClass::Link => self.append(":link")?,
                        CssPseudoClass::Visited => self.append(":visited")?,
                        CssPseudoClass::Target => self.append(":target")?,
                        CssPseudoClass::Hover => self.append(":hover")?,
                        CssPseudoClass::Active => self.append(":active")?,
                        CssPseudoClass::Focus => self.append(":focus")?,
                        CssPseudoClass::FocusVisible => self.append(":focus-visible")?,
                        CssPseudoClass::FocusWithin => self.append(":focus-within")?,
                        CssPseudoClass::Disabled => self.append(":disabled")?,
                        CssPseudoClass::Enabled => self.append(":enabled")?,
                        CssPseudoClass::Checked => self.append(":checked")?,
                        CssPseudoClass::Required => self.append(":required")?,
                        CssPseudoClass::Optional => self.append(":optional")?,
                        CssPseudoClass::Valid => self.append(":valid")?,
                        CssPseudoClass::Invalid => self.append(":invalid")?,
                        CssPseudoClass::PlaceholderShown => self.append(":placeholder-shown")?,
                        CssPseudoClass::FirstChild => self.append(":first-child")?,
                        CssPseudoClass::LastChild => self.append(":last-child")?,
                        CssPseudoClass::OnlyChild => self.append(":only-child")?,
                        CssPseudoClass::Empty => self.append(":empty")?,
                        CssPseudoClass::FirstOfType => self.append(":first-of-type")?,
                        CssPseudoClass::LastOfType => self.append(":last-of-type")?,
                        CssPseudoClass::OnlyOfType => self.append(":only-of-type")?,
                        CssPseudoClass::Modal => self.append(":modal")?,
                        CssPseudoClass::Fullscreen => self.append(":fullscreen")?,
                        CssPseudoClass::PopoverOpen => self.append(":popover-open")?,
                        CssPseudoClass::Default => self.append(":default")?,
                        CssPseudoClass::Indeterminate => self.append(":indeterminate")?,
                        CssPseudoClass::ReadOnly => self.append(":read-only")?,
                        CssPseudoClass::ReadWrite => self.append(":read-write")?,
                        CssPseudoClass::InRange => self.append(":in-range")?,
                        CssPseudoClass::OutOfRange => self.append(":out-of-range")?,
                    }
                }
                Event::Element(value) => {
                    self.node()?;
                    match value {
                        CssPseudoElement::Slotted(arg) => {
                            self.append("::slotted(")?;
                            push(&mut work, Event::Text(")"))?;
                            push(&mut work, Event::Compound(arg.compound()))?;
                        }
                        CssPseudoElement::Part(list) => {
                            self.append("::part(")?;
                            for (i, name) in list.names().iter().enumerate() {
                                if i != 0 {
                                    self.append(" ")?;
                                }
                                self.node()?;
                                self.selector_identifier(name.as_str())?;
                            }
                            self.append(")")?;
                        }
                        CssPseudoElement::Before => self.append("::before")?,
                        CssPseudoElement::After => self.append("::after")?,
                        CssPseudoElement::FirstLine => self.append("::first-line")?,
                        CssPseudoElement::FirstLetter => self.append("::first-letter")?,
                        CssPseudoElement::Marker => self.append("::marker")?,
                        CssPseudoElement::Selection => self.append("::selection")?,
                        CssPseudoElement::Backdrop => self.append("::backdrop")?,
                    }
                }
            }
        }
        Ok(())
    }
}
