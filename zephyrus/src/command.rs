use crate::{
    argument::CommandArgument, context::SlashContext, twilight_exports::Permissions, BoxFuture,
};
use std::collections::HashMap;
use std::error::Error;
use crate::hook::{CheckHook, ErrorHook};

/// The result of a command execution.
pub type CommandResult = Result<(), Box<dyn Error + Send + Sync>>;
/// A pointer to a command function.
pub(crate) type CommandFn<D> = for<'a> fn(&'a SlashContext<'a, D>) -> BoxFuture<'a, CommandResult>;
/// A map of [commands](self::Command).
pub type CommandMap<D> = HashMap<&'static str, Command<D>>;

/// A command executed by the framework.
pub struct Command<D> {
    /// The name of the command.
    pub name: &'static str,
    /// The description of the commands.
    pub description: &'static str,
    /// All the arguments the command requires.
    pub arguments: Vec<CommandArgument<D>>,
    /// A pointer to this command function.
    pub fun: CommandFn<D>,
    /// The required permissions to use this command
    pub required_permissions: Option<Permissions>,
    pub checks: Vec<CheckHook<D>>,
    pub error_handler: Option<ErrorHook<D>>
}

impl<D> Command<D> {
    /// Creates a new command.
    pub fn new(fun: CommandFn<D>) -> Self {
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

    pub fn checks(mut self, checks: Vec<CheckHook<D>>) -> Self {
        self.checks = checks;
        self
    }

    pub fn error_handler(mut self, hook: ErrorHook<D>) -> Self {
        self.error_handler = Some(hook);
        self
    }

    pub fn required_permissions(mut self, permissions: Permissions) -> Self {
        self.required_permissions = Some(permissions);
        self
    }

    pub async fn run_checks(&self, context: &SlashContext<'_, D>) -> bool {
        for check in &self.checks {
            if !(check.0)(context).await {
                return false;
            }
        }
        true
    }

    pub async fn execute(&self, context: &SlashContext<'_, D>) -> Option<CommandResult> {
        if self.run_checks(context).await {
            let output = (self.fun)(context).await;
            if let Some(hook) = &self.error_handler {
                (hook.0)(context, output).await;

                None
            } else {
                Some(output)
            }
        } else {
            None
        }
    }
}
