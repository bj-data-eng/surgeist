//! Iterative value operations on authored layer sequences.

use super::{Command, LayerCommands};
use crate::Layer;
use std::{
    fmt::{self, Write},
    mem, slice,
};

struct CloneFrame<'a> {
    layer: &'a Layer,
    siblings: slice::Iter<'a, Command>,
    output: Vec<Command>,
}

impl Clone for LayerCommands {
    fn clone(&self) -> Self {
        let mut commands = self.iter();
        let mut output = Vec::with_capacity(commands.len());
        let mut pending = Vec::new();
        loop {
            match commands.next() {
                Some(Command::Layer { layer, children }) if !children.is_empty() => {
                    let child_output = Vec::with_capacity(children.len());
                    pending.push(CloneFrame {
                        layer,
                        siblings: commands,
                        output: mem::replace(&mut output, child_output),
                    });
                    commands = children.iter();
                }
                Some(command) => output.push(command.clone()),
                None => {
                    let Some(CloneFrame {
                        layer,
                        siblings,
                        output: mut parent,
                    }) = pending.pop()
                    else {
                        return Self(output);
                    };
                    parent.push(Command::Layer {
                        layer: layer.clone(),
                        children: Self(output),
                    });
                    output = parent;
                    commands = siblings;
                }
            }
        }
    }
}

impl PartialEq for LayerCommands {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        let mut pairs = self.iter().zip(other.iter());
        let mut pending = Vec::new();
        loop {
            let Some((left, right)) = pairs.next() else {
                let Some(siblings) = pending.pop() else {
                    return true;
                };
                pairs = siblings;
                continue;
            };
            match (left, right) {
                (
                    Command::Layer { layer, children },
                    Command::Layer {
                        layer: other_layer,
                        children: other_children,
                    },
                ) => {
                    if layer != other_layer || children.len() != other_children.len() {
                        return false;
                    }
                    if !children.is_empty() {
                        if pairs.len() != 0 {
                            pending.push(pairs);
                        }
                        pairs = children.iter().zip(other_children.iter());
                    }
                }
                _ if left != right => return false,
                _ => {}
            }
        }
    }
}

impl fmt::Debug for LayerCommands {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return formatter.write_str("[]");
        }
        let mut output = DebugTree::new(formatter);
        let mut commands = self.iter();
        let mut pending = Vec::new();
        output.open("[", "[\n")?;
        loop {
            match commands.next() {
                Some(Command::Layer { layer, children }) => {
                    output.open("Layer { ", "Layer {\n")?;
                    output.write_str("layer: ")?;
                    output.value(layer)?;
                    output.write_str(if output.pretty {
                        ",\nchildren: "
                    } else {
                        ", children: "
                    })?;
                    if children.is_empty() {
                        output.write_str("[]")?;
                        output.close_layer()?;
                        output.finish_entry(!commands.as_slice().is_empty())?;
                    } else {
                        output.open("[", "[\n")?;
                        pending.push(commands);
                        commands = children.iter();
                    }
                }
                Some(command) => {
                    output.value(command)?;
                    output.finish_entry(!commands.as_slice().is_empty())?;
                }
                None => {
                    output.close("]")?;
                    let Some(siblings) = pending.pop() else {
                        return Ok(());
                    };
                    commands = siblings;
                    output.close_layer()?;
                    output.finish_entry(!commands.as_slice().is_empty())?;
                }
            }
        }
    }
}

struct DebugTree<'a, 'f> {
    formatter: &'a mut fmt::Formatter<'f>,
    pretty: bool,
    indent: usize,
    line_start: bool,
}

impl<'a, 'f> DebugTree<'a, 'f> {
    fn new(formatter: &'a mut fmt::Formatter<'f>) -> Self {
        Self {
            pretty: formatter.alternate(),
            formatter,
            indent: 0,
            line_start: false,
        }
    }

    fn open(&mut self, compact: &str, pretty: &str) -> fmt::Result {
        if self.pretty {
            self.write_str(pretty)?;
            self.indent = self.indent.checked_add(1).ok_or(fmt::Error)?;
            Ok(())
        } else {
            self.write_str(compact)
        }
    }

    fn close(&mut self, text: &str) -> fmt::Result {
        if self.pretty {
            self.indent = self.indent.checked_sub(1).ok_or(fmt::Error)?;
        }
        self.write_str(text)
    }

    fn close_layer(&mut self) -> fmt::Result {
        self.write_str(if self.pretty { ",\n" } else { " " })?;
        self.close("}")
    }

    fn finish_entry(&mut self, more: bool) -> fmt::Result {
        if self.pretty {
            self.write_str(",\n")
        } else if more {
            self.write_str(", ")
        } else {
            Ok(())
        }
    }

    fn value(&mut self, value: &impl fmt::Debug) -> fmt::Result {
        if self.pretty {
            // Use ordinary alternate formatting through the indentation
            // writer. Stable Formatter cannot transfer its extra flags.
            write!(self, "{value:#?}")
        } else {
            value.fmt(self.formatter)
        }
    }
}

impl Write for DebugTree<'_, '_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        if !self.pretty {
            return self.formatter.write_str(text);
        }
        const PADDING: &str = "                                                                ";
        const LEVELS: usize = PADDING.len() / 4;
        for line in text.split_inclusive('\n') {
            if self.line_start {
                for _ in 0..self.indent / LEVELS {
                    self.formatter.write_str(PADDING)?;
                }
                let width = (self.indent % LEVELS) * 4;
                self.formatter
                    .write_str(PADDING.get(..width).ok_or(fmt::Error)?)?;
            }
            self.line_start = line.ends_with('\n');
            self.formatter.write_str(line)?;
        }
        Ok(())
    }
}
