use super::{
    Command, CommandKind, Error, Id, ImeConfig, ImeRequest, Result,
    geometry::{normalize_local_rect, normalize_nonnegative_size, normalize_outer_position},
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NormalizedCommand {
    command: Command,
}

impl NormalizedCommand {
    #[must_use]
    pub(crate) fn command(&self) -> &Command {
        &self.command
    }

    #[must_use]
    pub(crate) fn into_command(self) -> Command {
        self.command
    }
}

/// Intrinsically validates and canonicalizes one authored command.
pub(crate) fn normalize_command(command: Command) -> Result<NormalizedCommand> {
    let kind = command.kind();
    let target = command.target();
    normalize_command_value(command)
        .map(|command| NormalizedCommand { command })
        .map_err(|error| command_error(error, kind, target))
}

/// Intrinsically validates and canonicalizes one ordered authored batch.
pub(crate) fn normalize_commands(commands: Vec<Command>) -> Result<Vec<NormalizedCommand>> {
    commands.into_iter().map(normalize_command).collect()
}

fn command_error(mut error: Error, kind: CommandKind, target: Option<Id>) -> Error {
    error = error.with_command_kind(kind);
    if let Some(id) = target {
        error = error.with_id(id);
    }
    error
}

fn normalize_command_value(command: Command) -> Result<Command> {
    match command {
        Command::Open { request } => Ok(Command::Open {
            request: request.normalize()?,
        }),
        Command::SetPosition { id, position } => Ok(Command::SetPosition {
            id,
            position: normalize_outer_position(position)?,
        }),
        Command::SetInnerSize { id, size } => Ok(Command::SetInnerSize {
            id,
            size: normalize_nonnegative_size(size, "inner size")?,
        }),
        Command::SetMinInnerSize { id, size } => Ok(Command::SetMinInnerSize {
            id,
            size: size
                .map(|size| normalize_nonnegative_size(size, "minimum inner size"))
                .transpose()?,
        }),
        Command::SetMaxInnerSize { id, size } => Ok(Command::SetMaxInnerSize {
            id,
            size: size
                .map(|size| normalize_nonnegative_size(size, "maximum inner size"))
                .transpose()?,
        }),
        Command::SetIme { id, request } => Ok(Command::SetIme {
            id,
            request: normalize_ime_request(request)?,
        }),
        #[cfg(feature = "accessibility")]
        Command::UpdateAccessibility { id, update } => {
            Ok(Command::UpdateAccessibility { id, update })
        }
        command => Ok(command),
    }
}

fn normalize_ime_request(request: ImeRequest) -> Result<ImeRequest> {
    match request {
        ImeRequest::Disable => Ok(ImeRequest::Disable),
        ImeRequest::Enable(config) => Ok(ImeRequest::Enable(normalize_ime_config(config)?)),
        ImeRequest::Update(config) => Ok(ImeRequest::Update(normalize_ime_config(config)?)),
        ImeRequest::Restart(config) => Ok(ImeRequest::Restart(normalize_ime_config(config)?)),
    }
}

fn normalize_ime_config(mut config: ImeConfig) -> Result<ImeConfig> {
    config.cursor_area = config.cursor_area.map(normalize_local_rect).transpose()?;
    Ok(config)
}
