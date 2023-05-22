use crate::{
    argument::CommandArgument, context::SlashContext, twilight_exports::Permissions, BoxFuture, framework::ProcessResult,
};
use std::collections::HashMap;
use tracing::{debug, info};
use crate::hook::{CheckHook, ErrorHandlerHook};

/// A pointer to a command function.
pub(crate) type CommandFn<D, T, E> = for<'a> fn(&'a SlashContext<'a, D>) -> BoxFuture<'a, Result<T, E>>;
/// A map of [commands](self::Command).
pub type CommandMap<D, T, E> = HashMap<&'static str, Command<D, T, E>>;

/// Information about the execution state of a command.
#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub enum ExecutionState {
    /// A check had an error.
    CheckErrored,
    /// A check returned `false` and the command didn't execute.
    CheckFailed,
    /// The command finished executing without errors.
    CommandFinished,
    /// The error handler raised an error. 
    CommandErrored,
    /// The `before` hook returned `false` and the command didn't execute.
    BeforeHookFailed
}

/// The location of the output of the command.
#[non_exhaustive]
pub enum OutputLocation<T, E> {
    /// The command was not executed, thus there is not any output.
    NotExecuted,
    /// The output has not been taken by any hook.
    Present(Result<T, E>),
    /// The output has been forwarded to the `after` hook.
    TakenByAfterHook,
    /// The output has been taken by the `error_handler` hook.
    TakenByErrorHandlerHook
}

/// Information about the command execution and it's output.
pub struct ExecutionResult<T, E> {
    /// The execution state of the command.
    pub state: ExecutionState,
    /// The output of the command.
    pub output: OutputLocation<T, E>
}

impl<T, E> From<ExecutionResult<T, E>> for ProcessResult<T, E> {
    fn from(value: ExecutionResult<T, E>) -> Self {
        ProcessResult::CommandExecuted(value)
    }
}

/// A command executed by the framework.
pub struct Command<D, T, E> {
    /// The name of the command.
    pub name: &'static str,
    /// The description of the commands.
    pub description: &'static str,
    /// All the arguments the command requires.
    pub arguments: Vec<CommandArgument<D>>,
    /// A pointer to this command function.
    pub fun: CommandFn<D, T, E>,
    /// The required permissions to use this command
    pub required_permissions: Option<Permissions>,
    pub checks: Vec<CheckHook<D, E>>,
    pub error_handler: Option<ErrorHandlerHook<D, E>>
}

impl<D, T, E> Command<D, T, E> {
    /// Creates a new command.
    pub fn new(fun: CommandFn<D, T, E>) -> Self {
        Self {
            name: Default::default(),
            description: Default::default(),
            arguments: Default::default(),
            fun,
            required_permissions: Default::default(),
            checks: Default::default(),
            error_handler: None
        }
    }

    /// Sets the command name.
    pub fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Sets the command description.
    pub fn description(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }

    /// Adds an argument to the command.
    pub fn add_argument(mut self, arg: CommandArgument<D>) -> Self {
        self.arguments.push(arg);
        self
    }

    pub fn checks(mut self, checks: Vec<CheckHook<D, E>>) -> Self {
        self.checks = checks;
        self
    }

    pub fn error_handler(mut self, hook: ErrorHandlerHook<D, E>) -> Self {
        self.error_handler = Some(hook);
        self
    }

    pub fn required_permissions(mut self, permissions: Permissions) -> Self {
        self.required_permissions = Some(permissions);
        self
    }

    pub async fn run_checks(&self, context: &SlashContext<'_, D>) -> Result<bool, E> {
        debug!("Running command [{}] checks", self.name);
        for check in &self.checks {
            if !(check.0)(context).await? {
                debug!("Command [{}] check returned false", self.name);
                return Ok(false);
            }
        }
        debug!("All command [{}] checks passed", self.name);
        Ok(true)
    }

    pub async fn execute(&self, context: &SlashContext<'_, D>) -> ExecutionResult<T, E> {
        let state;
        let location;

        match self.run_checks(context).await {
            Ok(true) => {
                debug!("Executing command [{}]", self.name);
                let output = (self.fun)(context).await;

                match (&self.error_handler, output) {
                    (Some(hook), Err(why)) => {
                        info!("Command [{}] raised an error, using established error handler", self.name);
                        state = ExecutionState::CommandErrored;
                        location = OutputLocation::TakenByErrorHandlerHook;

                        (hook.0)(context, why).await;
                    },
                    (_, Ok(res)) => {
                        debug!("Command [{}] executed successfully", self.name);
                        state = ExecutionState::CommandFinished;
                        location = OutputLocation::Present(Ok(res));
                    },
                    (_, Err(res)) => {
                        info!("Command [{}] raised an error, but no error handler was established", self.name);
                        state = ExecutionState::CommandErrored;
                        location = OutputLocation::Present(Err(res));
                    }
                };
            },
            Err(why) => {
                state = ExecutionState::CheckErrored;
                // If the command has an error handler, execute it, if not, discard the error.
                if let Some(hook) = &self.error_handler {
                    info!("Command [{}] check raised an error, using established error handler", self.name);
                    (hook.0)(context, why).await;
                    location = OutputLocation::TakenByErrorHandlerHook;
                } else {
                    info!("Command [{}] check raised an error, but no error handler was established", self.name);
                    location = OutputLocation::Present(Err(why));
                }
            },
            _ => {
                state = ExecutionState::CheckFailed;
                location = OutputLocation::NotExecuted;
            }
        }

        ExecutionResult {
            state,
            output: location
        }
    }
}
