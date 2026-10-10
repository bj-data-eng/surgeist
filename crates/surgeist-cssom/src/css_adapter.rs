//! CSS-owned checked syntax enters one live owner without text reconstruction.
use crate::{model::State, *};
use surgeist_css::*;

impl State {
    pub(crate) fn allocate(&mut self, limits: &CssomLimits) -> Result<u64, CssomError> {
        let value = self.next_identity;
        if value == 0 || value > limits.max_identity {
            return Err(CssomError::IdentityExhausted);
        }
        self.next_identity = value.checked_add(1).unwrap_or(0);
        Ok(value)
    }
    pub(crate) fn list(&mut self, limits: &CssomLimits) -> Result<CssomRuleListId, CssomError> {
        let id = CssomRuleListId {
            owner: self.owner.clone(),
            serial: self.allocate(limits)?,
        };
        self.lists.insert(id.clone(), Vec::new());
        Ok(id)
    }
    pub(crate) fn media_list(
        &mut self,
        value: CssMediaQueryList,
        limits: &CssomLimits,
    ) -> Result<CssomMediaListId, CssomError> {
        let id = CssomMediaListId {
            owner: self.owner.clone(),
            serial: self.allocate(limits)?,
        };
        self.media.insert(id.clone(), value);
        Ok(id)
    }
    pub(crate) fn block_record(
        &mut self,
        data: CssomBlockData,
        parent: Option<CssomRuleId>,
        owner: Option<CssomInputVersion>,
        flags: CssomBlockFlags,
        limits: &CssomLimits,
    ) -> Result<CssomBlockId, CssomError> {
        let id = CssomBlockId {
            owner: self.owner.clone(),
            serial: self.allocate(limits)?,
        };
        self.blocks.insert(
            id.clone(),
            CssomBlock {
                parent,
                owner,
                flags,
                data,
            },
        );
        Ok(id)
    }
    fn ordinary_block(
        &mut self,
        declarations: &CssDeclarationList,
        parent: &CssomRuleId,
        limits: &CssomLimits,
    ) -> Result<CssomBlockId, CssomError> {
        check_declarations(declarations.iter(), limits)?;
        let selected = projection(
            CssSpecifiedDeclarationBlock::try_from_declarations_with_limits(
                declarations,
                limits.css,
            ),
        )?;
        self.block_record(
            CssomBlockData::Properties {
                domain: CssomPropertyDomain::Ordinary,
                authored: CssomPropertyOccurrences::Ordinary(declarations.clone()),
                selected,
            },
            Some(parent.clone()),
            None,
            CssomBlockFlags::default(),
            limits,
        )
    }
    fn ordinary_children(
        &mut self,
        rules: &[CssRule],
        parent: &CssomRuleId,
        depth: usize,
        limits: &CssomLimits,
    ) -> Result<CssomRuleListId, CssomError> {
        let list = self.list(limits)?;
        for rule in rules {
            let id = self.adopt(
                CssomAuthoredRule::Ordinary(rule.clone()),
                CssomParent::Rule(parent.clone()),
                depth + 1,
                limits,
            )?;
            self.lists.get_mut(&list).expect("new list").push(id);
        }
        Ok(list)
    }
    fn scoped_children(
        &mut self,
        rules: &[CssScopedRule],
        parent: &CssomRuleId,
        depth: usize,
        limits: &CssomLimits,
    ) -> Result<CssomRuleListId, CssomError> {
        let list = self.list(limits)?;
        for rule in rules {
            let id = self.adopt(
                CssomAuthoredRule::Scoped(rule.clone()),
                CssomParent::Rule(parent.clone()),
                depth + 1,
                limits,
            )?;
            self.lists.get_mut(&list).expect("new list").push(id);
        }
        Ok(list)
    }
    pub(crate) fn adopt(
        &mut self,
        authored: CssomAuthoredRule,
        parent: CssomParent,
        depth: usize,
        limits: &CssomLimits,
    ) -> Result<CssomRuleId, CssomError> {
        if depth > limits.max_depth {
            return Err(CssomError::Limit {
                resource: "rule depth",
                maximum: limits.max_depth,
            });
        }
        let id = CssomRuleId {
            owner: self.owner.clone(),
            serial: self.allocate(limits)?,
        };
        let data = match &authored {
            CssomAuthoredRule::Ordinary(v) => self.ordinary_data(v, &id, depth, limits)?,
            CssomAuthoredRule::Scoped(v) => self.scoped_data(v, &id, depth, limits)?,
            CssomAuthoredRule::Keyframe(v) => {
                check_declarations(v.declarations().iter().map(|v| v.source()), limits)?;
                let selected = projection(
                    CssSpecifiedDeclarationBlock::try_from_keyframe_declarations_with_limits(
                        v.declarations(),
                        limits.css,
                    ),
                )?;
                let block = self.block_record(
                    CssomBlockData::Properties {
                        domain: CssomPropertyDomain::Keyframe,
                        authored: CssomPropertyOccurrences::Keyframe(v.declarations().clone()),
                        selected,
                    },
                    Some(id.clone()),
                    None,
                    CssomBlockFlags::default(),
                    limits,
                )?;
                CssomRuleData::Keyframe {
                    selectors: v.selectors().clone(),
                    block,
                }
            }
            CssomAuthoredRule::Margin(v) => {
                check_declarations(v.declarations().properties().iter(), limits)?;
                let selected = projection(
                    v.declarations()
                        .try_specified_properties_with_limits(limits.css),
                )?;
                let block = self.block_record(
                    CssomBlockData::Properties {
                        domain: CssomPropertyDomain::Margin,
                        authored: CssomPropertyOccurrences::Margin(v.declarations().clone()),
                        selected,
                    },
                    Some(id.clone()),
                    None,
                    CssomBlockFlags::default(),
                    limits,
                )?;
                CssomRuleData::Margin {
                    name: v.name(),
                    block,
                }
            }
        };
        self.rules.insert(
            id.clone(),
            CssomRule {
                parent,
                data,
                authored,
            },
        );
        self.check_limits(limits)?;
        Ok(id)
    }
    fn page_data(
        &mut self,
        v: &CssPageRule,
        id: &CssomRuleId,
        depth: usize,
        limits: &CssomLimits,
    ) -> Result<CssomRuleData, CssomError> {
        check_declarations(v.declarations().properties().iter(), limits)?;
        let selected = match v.declarations().try_specified_with_limits(limits.css) {
            Ok(v) => CssomProjection::Available(v),
            Err(e) if page_resource(&e) => return Err(CssomError::Page(e)),
            Err(e) => CssomProjection::Unavailable(e),
        };
        let block = self.block_record(
            CssomBlockData::Page {
                authored: v.declarations().clone(),
                selected,
            },
            Some(id.clone()),
            None,
            CssomBlockFlags::default(),
            limits,
        )?;
        let children = self.list(limits)?;
        for child in v.margin_rules() {
            let child = self.adopt(
                CssomAuthoredRule::Margin(child.clone()),
                CssomParent::Rule(id.clone()),
                depth + 1,
                limits,
            )?;
            self.lists.get_mut(&children).expect("new list").push(child);
        }
        Ok(CssomRuleData::Page {
            selectors: v.selectors().clone(),
            block,
            children,
        })
    }
    fn keyframes_data(
        &mut self,
        v: &CssKeyframesRule,
        id: &CssomRuleId,
        depth: usize,
        limits: &CssomLimits,
    ) -> Result<CssomRuleData, CssomError> {
        let children = self.list(limits)?;
        for child in v.blocks() {
            let child = self.adopt(
                CssomAuthoredRule::Keyframe(child.clone()),
                CssomParent::Rule(id.clone()),
                depth + 1,
                limits,
            )?;
            self.lists.get_mut(&children).expect("new list").push(child);
        }
        Ok(CssomRuleData::Keyframes {
            name: v.name().clone(),
            children,
        })
    }
    fn counter_data(
        &mut self,
        v: &CssCounterStyleRule,
        id: &CssomRuleId,
        limits: &CssomLimits,
    ) -> Result<CssomRuleData, CssomError> {
        let block = self.block_record(
            CssomBlockData::CounterStyle(Box::new(v.descriptors().clone())),
            Some(id.clone()),
            None,
            CssomBlockFlags::default(),
            limits,
        )?;
        Ok(CssomRuleData::CounterStyle {
            name: v.name().as_str().to_owned(),
            block,
        })
    }
    fn font_data(
        &mut self,
        v: &CssFontFaceRule,
        id: &CssomRuleId,
        limits: &CssomLimits,
    ) -> Result<CssomRuleData, CssomError> {
        let block = self.block_record(
            CssomBlockData::FontFace(v.descriptors().clone()),
            Some(id.clone()),
            None,
            CssomBlockFlags::default(),
            limits,
        )?;
        Ok(CssomRuleData::FontFace { block })
    }
    fn features_data(
        &mut self,
        v: &CssFontFeatureValuesRule,
        limits: &CssomLimits,
    ) -> Result<CssomRuleData, CssomError> {
        let mut maps = Vec::new();
        for kind in [
            CssFontFeatureValueKind::Stylistic,
            CssFontFeatureValueKind::HistoricalForms,
            CssFontFeatureValueKind::Styleset,
            CssFontFeatureValueKind::CharacterVariant,
            CssFontFeatureValueKind::Swash,
            CssFontFeatureValueKind::Ornaments,
            CssFontFeatureValueKind::Annotation,
        ] {
            let id = CssomFeatureMapId {
                owner: self.owner.clone(),
                serial: self.allocate(limits)?,
            };
            let mut map = CssomFeatureMap {
                kind,
                entries: Vec::new(),
                authored: Vec::new(),
            };
            for item in v.items() {
                if let CssFontFeatureValuesItem::Block(block) = item {
                    if block.kind() != kind {
                        continue;
                    }
                    for definition in block.definitions() {
                        map.authored.push(definition.clone());
                        let value = match definition.value().view() {
                            CssFontFeatureValueRef::Indexes(indexes) => indexes
                                .iter()
                                .map(CssFontFeatureValueIndex::to_u32)
                                .collect::<Option<Vec<_>>>()
                                .map_or_else(
                                    || CssomFeatureValue::Authored(definition.value().clone()),
                                    CssomFeatureValue::Unsigned,
                                ),
                            _ => CssomFeatureValue::Authored(definition.value().clone()),
                        };
                        let entry = CssomFeatureEntry {
                            key: definition.name().as_str().to_owned(),
                            value,
                        };
                        if let Some(old) = map.entries.iter_mut().find(|v| v.key == entry.key) {
                            *old = entry;
                        } else {
                            map.entries.push(entry);
                        }
                    }
                }
            }
            self.maps.insert(id.clone(), map);
            maps.push(id);
        }
        Ok(CssomRuleData::FontFeatureValues {
            families: v.families().to_vec(),
            maps,
            font_display: v
                .items()
                .iter()
                .filter_map(|v| match v {
                    CssFontFeatureValuesItem::FontDisplay(v) => Some(v.clone()),
                    _ => None,
                })
                .collect(),
        })
    }
    fn ordinary_data(
        &mut self,
        v: &CssRule,
        id: &CssomRuleId,
        depth: usize,
        limits: &CssomLimits,
    ) -> Result<CssomRuleData, CssomError> {
        Ok(match v {
            CssRule::Style(v) => CssomRuleData::Style {
                selectors: CssomSelectors::Ordinary(v.selectors().clone()),
                block: self.ordinary_block(v.declarations(), id, limits)?,
                children: self.ordinary_children(v.rules(), id, depth, limits)?,
            },
            CssRule::NestedDeclarations(v) => CssomRuleData::NestedDeclarations {
                block: self.ordinary_block(v.declarations(), id, limits)?,
            },
            CssRule::Media(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::Media(self.media_list(v.query().clone(), limits)?),
                children: self.ordinary_children(v.rules(), id, depth, limits)?,
            },
            CssRule::Supports(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::Supports(v.condition().clone()),
                children: self.ordinary_children(v.rules(), id, depth, limits)?,
            },
            CssRule::Container(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::Container(v.prelude().clone()),
                children: self.ordinary_children(v.rules(), id, depth, limits)?,
            },
            CssRule::LayerBlock(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::Layer(v.name().cloned()),
                children: self.ordinary_children(v.rules(), id, depth, limits)?,
            },
            CssRule::When(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::When(v.condition().clone()),
                children: self.ordinary_children(v.rules(), id, depth, limits)?,
            },
            CssRule::Else(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::Else(v.condition().cloned()),
                children: self.ordinary_children(v.rules(), id, depth, limits)?,
            },
            CssRule::Scope(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::Scope {
                    start: v.root().cloned(),
                    end: v.limit().cloned(),
                },
                children: self.scoped_children(v.rules().rules(), id, depth, limits)?,
            },
            CssRule::Page(v) => self.page_data(v, id, depth, limits)?,
            CssRule::Keyframes(v) => self.keyframes_data(v, id, depth, limits)?,
            CssRule::CounterStyle(v) => self.counter_data(v, id, limits)?,
            CssRule::FontFace(v) => self.font_data(v, id, limits)?,
            CssRule::FontFeatureValues(v) => self.features_data(v, limits)?,
            CssRule::Import(v) => CssomRuleData::Import {
                target: v.target().clone(),
                layer: v.layer().cloned(),
                supports: v.supports().cloned(),
                origin: v.origin().clone(),
                media: self.media_list(
                    v.media()
                        .cloned()
                        .unwrap_or_else(|| CssMediaQueryList::new(Vec::new())),
                    limits,
                )?,
                child_sheet: None,
                resolved_location: None,
                input: None,
            },
            _ => CssomRuleData::Leaf {
                current: CssomAuthoredRule::Ordinary(v.clone()),
            },
        })
    }
    fn scoped_data(
        &mut self,
        v: &CssScopedRule,
        id: &CssomRuleId,
        depth: usize,
        limits: &CssomLimits,
    ) -> Result<CssomRuleData, CssomError> {
        Ok(match v {
            CssScopedRule::Style(v) => CssomRuleData::Style {
                selectors: CssomSelectors::Scoped(v.selectors().clone()),
                block: self.ordinary_block(v.declarations(), id, limits)?,
                children: self.ordinary_children(v.rules(), id, depth, limits)?,
            },
            CssScopedRule::NestedDeclarations(v) => CssomRuleData::NestedDeclarations {
                block: self.ordinary_block(v.declarations(), id, limits)?,
            },
            CssScopedRule::Media(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::Media(self.media_list(v.query().clone(), limits)?),
                children: self.scoped_children(v.rules().rules(), id, depth, limits)?,
            },
            CssScopedRule::Supports(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::Supports(v.condition().clone()),
                children: self.scoped_children(v.rules().rules(), id, depth, limits)?,
            },
            CssScopedRule::Container(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::Container(v.prelude().clone()),
                children: self.scoped_children(v.rules().rules(), id, depth, limits)?,
            },
            CssScopedRule::LayerBlock(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::Layer(v.name().cloned()),
                children: self.scoped_children(v.rules().rules(), id, depth, limits)?,
            },
            CssScopedRule::When(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::When(v.condition().clone()),
                children: self.scoped_children(v.rules().rules(), id, depth, limits)?,
            },
            CssScopedRule::Else(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::Else(v.condition().cloned()),
                children: self.scoped_children(v.rules().rules(), id, depth, limits)?,
            },
            CssScopedRule::Scope(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::Scope {
                    start: v.root().cloned(),
                    end: v.limit().cloned(),
                },
                children: self.scoped_children(v.rules().rules(), id, depth, limits)?,
            },
            CssScopedRule::Page(v) => self.page_data(v, id, depth, limits)?,
            CssScopedRule::Keyframes(v) => self.keyframes_data(v, id, depth, limits)?,
            CssScopedRule::CounterStyle(v) => self.counter_data(v, id, limits)?,
            CssScopedRule::FontFace(v) => self.font_data(v, id, limits)?,
            CssScopedRule::FontFeatureValues(v) => self.features_data(v, limits)?,
            _ => CssomRuleData::Leaf {
                current: CssomAuthoredRule::Scoped(v.clone()),
            },
        })
    }
}

pub(crate) fn resource_kind(kind: CssSpecifiedValueSerializationErrorKind) -> bool {
    matches!(
        kind,
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
            | CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
            | CssSpecifiedValueSerializationErrorKind::ByteLimit
            | CssSpecifiedValueSerializationErrorKind::CapacityOverflow
    )
}
pub(crate) fn declaration_resource(e: &CssDeclarationBlockError) -> bool {
    matches!(e.kind(),CssDeclarationBlockErrorKind::Serialization(e) if resource_kind(e.kind()))
}
fn page_resource(e: &CssPageProjectionError) -> bool {
    match e {
        CssPageProjectionError::Resource(e) => resource_kind(e.kind()),
        CssPageProjectionError::Properties(e) => declaration_resource(e),
        _ => false,
    }
}
pub(crate) fn projection(
    result: Result<CssSpecifiedDeclarationBlock, CssDeclarationBlockError>,
) -> Result<CssomProjection<CssSpecifiedDeclarationBlock, CssDeclarationBlockError>, CssomError> {
    match result {
        Ok(v) => Ok(CssomProjection::Available(v)),
        Err(e) if declaration_resource(&e) => Err(CssomError::Declaration(e)),
        Err(e) => Ok(CssomProjection::Unavailable(e)),
    }
}
pub(crate) fn check_declarations<'a>(
    declarations: impl IntoIterator<Item = &'a CssDeclaration>,
    limits: &CssomLimits,
) -> Result<(), CssomError> {
    let mut nodes = 0usize;
    for declaration in declarations {
        let values = declaration.value_components();
        nodes = nodes
            .checked_add(values.component_count())
            .filter(|v| *v <= limits.max_entries)
            .ok_or(CssomError::Limit {
                resource: "authored component entries",
                maximum: limits.max_entries,
            })?;
        if declaration
            .parsed_value()
            .is_some_and(|v| v.source().as_str().len() > limits.max_input_bytes)
        {
            return Err(CssomError::Limit {
                resource: "retained parse input bytes",
                maximum: limits.max_input_bytes,
            });
        }
        // Charge raw component output even when symbolic expansion cannot select
        // a footprint. No semantic wrapper or parse round trip is involved.
        if let Err(e) = values.serialize_with_limit(limits.max_input_bytes)
            && matches!(
                e.kind(),
                CssComponentValueErrorKind::NestingLimit
                    | CssComponentValueErrorKind::ComponentLimit
                    | CssComponentValueErrorKind::ByteLimit
                    | CssComponentValueErrorKind::CapacityOverflow
            )
        {
            return Err(CssomError::Component(e));
        }
    }
    Ok(())
}
pub(crate) fn parse(
    source: &str,
    limits: &CssomLimits,
    context: CssParserContext,
) -> Result<(CssSheet, Vec<CssRecoveryDiagnostic>), CssomError> {
    if source.len() > limits.max_input_bytes {
        return Err(CssomError::Limit {
            resource: "parse input bytes",
            maximum: limits.max_input_bytes,
        });
    }
    let report = context.parse_sheet(source);
    for diagnostic in report.diagnostics() {
        if matches!(diagnostic.error().kind(), ErrorKind::NestingLimit(_))
            || matches!(diagnostic.error().kind(),ErrorKind::InvalidComponentValue(e) if matches!(e.kind(),CssComponentValueErrorKind::NestingLimit|CssComponentValueErrorKind::ComponentLimit|CssComponentValueErrorKind::ByteLimit|CssComponentValueErrorKind::CapacityOverflow))
        {
            return Err(CssomError::ParseResource(Box::new(diagnostic.clone())));
        }
    }
    Ok(report.into_parts())
}
