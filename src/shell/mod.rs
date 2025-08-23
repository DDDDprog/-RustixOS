// RustixOS Advanced Shell System
// Professional shell with full POSIX-like features

pub mod commands;
pub mod builtins;
pub mod parser;
pub mod completion;
pub mod history;
pub mod job_control;
pub mod variables;
pub mod aliases;
pub mod prompt;

use alloc::{string::String, vec::Vec, collections::BTreeMap};
use core::fmt;
use crate::println;

pub use commands::*;
pub use builtins::*;
pub use parser::*;
pub use completion::*;
pub use history::*;

/// Shell configuration
#[derive(Debug, Clone)]
pub struct ShellConfig {
    pub prompt: String,
    pub history_size: usize,
    pub completion_enabled: bool,
    pub job_control_enabled: bool,
    pub aliases_enabled: bool,
    pub variables_enabled: bool,
    pub color_enabled: bool,
    pub vi_mode: bool,
    pub auto_cd: bool,
    pub glob_enabled: bool,
}

impl Default for ShellConfig {
    fn default() -> Self {
        ShellConfig {
            prompt: "rustix$ ".to_string(),
            history_size: 1000,
            completion_enabled: true,
            job_control_enabled: true,
            aliases_enabled: true,
            variables_enabled: true,
            color_enabled: true,
            vi_mode: false,
            auto_cd: true,
            glob_enabled: true,
        }
    }
}

/// Shell state
pub struct Shell {
    config: ShellConfig,
    history: History,
    variables: variables::Variables,
    aliases: aliases::Aliases,
    jobs: job_control::JobManager,
    current_dir: String,
    exit_code: i32,
    running: bool,
}

/// Shell error types
#[derive(Debug, Clone)]
pub enum ShellError {
    CommandNotFound(String),
    InvalidSyntax(String),
    PermissionDenied(String),
    FileNotFound(String),
    DirectoryNotFound(String),
    InvalidArgument(String),
    IoError(String),
    NetworkError(String),
    BluetoothError(String),
    WifiError(String),
    SystemError(String),
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ShellError::CommandNotFound(cmd) => write!(f, "command not found: {}", cmd),
            ShellError::InvalidSyntax(msg) => write!(f, "syntax error: {}", msg),
            ShellError::PermissionDenied(msg) => write!(f, "permission denied: {}", msg),
            ShellError::FileNotFound(file) => write!(f, "file not found: {}", file),
            ShellError::DirectoryNotFound(dir) => write!(f, "directory not found: {}", dir),
            ShellError::InvalidArgument(arg) => write!(f, "invalid argument: {}", arg),
            ShellError::IoError(msg) => write!(f, "I/O error: {}", msg),
            ShellError::NetworkError(msg) => write!(f, "network error: {}", msg),
            ShellError::BluetoothError(msg) => write!(f, "bluetooth error: {}", msg),
            ShellError::WifiError(msg) => write!(f, "wifi error: {}", msg),
            ShellError::SystemError(msg) => write!(f, "system error: {}", msg),
        }
    }
}

pub type ShellResult<T> = Result<T, ShellError>;

impl Shell {
    pub fn new() -> Self {
        Shell {
            config: ShellConfig::default(),
            history: History::new(1000),
            variables: variables::Variables::new(),
            aliases: aliases::Aliases::new(),
            jobs: job_control::JobManager::new(),
            current_dir: "/".to_string(),
            exit_code: 0,
            running: true,
        }
    }

    pub fn with_config(config: ShellConfig) -> Self {
        let history_size = config.history_size;
        Shell {
            config,
            history: History::new(history_size),
            variables: variables::Variables::new(),
            aliases: aliases::Aliases::new(),
            jobs: job_control::JobManager::new(),
            current_dir: "/".to_string(),
            exit_code: 0,
            running: true,
        }
    }

    pub fn run(&mut self) {
        self.print_welcome();
        self.setup_environment();

        while self.running {
            let prompt = self.generate_prompt();
            print!("{}", prompt);

            if let Some(input) = self.read_line() {
                if !input.trim().is_empty() {
                    self.history.add(input.clone());
                    
                    match self.execute_line(&input) {
                        Ok(exit_code) => self.exit_code = exit_code,
                        Err(e) => {
                            println!("rustix: {}", e);
                            self.exit_code = 1;
                        }
                    }
                }
            }
        }
    }
    workspace
    fn print_welcome(&self) {
        println!("Welcome to RustixOS Shell v1.0");
        println!("Type 'help' for available commands or 'exit' to quit.");
        println!();
    }

    fn setup_environment(&mut self) {
        // Set default environment variables
        self.variables.set("PATH".to_string(), "/bin:/usr/bin:/usr/local/bin".to_string());
        self.variables.set("HOME".to_string(), "/home/user".to_string());
        self.variables.set("USER".to_string(), "user".to_string());
        self.variables.set("SHELL".to_string(), "/bin/rustix-shell".to_string());
        self.variables.set("PWD".to_string(), self.current_dir.clone());
        self.variables.set("TERM".to_string(), "xterm-256color".to_string());
        
        // Set default aliases
        self.aliases.set("ll".to_string(), "ls -la".to_string());
        self.aliases.set("la".to_string(), "ls -a".to_string());
        self.aliases.set("l".to_string(), "ls -CF".to_string());
        self.aliases.set("..".to_string(), "cd ..".to_string());
        self.aliases.set("...".to_string(), "cd ../..".to_string());
        self.aliases.set("grep".to_string(), "grep --color=auto".to_string());
        self.aliases.set("fgrep".to_string(), "fgrep --color=auto".to_string());
        self.aliases.set("egrep".to_string(), "egrep --color=auto".to_string());
    }

    fn generate_prompt(&self) -> String {
        if self.config.color_enabled {
            format!("\x1b[32m{}@rustix\x1b[0m:\x1b[34m{}\x1b[0m$ ", 
                   self.variables.get("USER").unwrap_or("user"), 
                   self.current_dir)
        } else {
            format!("{}@rustix:{}$ ", 
                   self.variables.get("USER").unwrap_or("user"), 
                   self.current_dir)
        }
    }

    fn read_line(&self) -> Option<String> {
        // This would integrate with keyboard input system
        // For now, return a placeholder
        Some("help".to_string())
    }

    pub fn execute_line(&mut self, line: &str) -> ShellResult<i32> {
        let line = line.trim();
        if line.is_empty() {
            return Ok(0);
        }

        // Parse the command line
        let parsed = self.parse_command_line(line)?;
        
        // Execute the parsed command
        self.execute_parsed_command(parsed)
    }

    fn parse_command_line(&self, line: &str) -> ShellResult<ParsedCommand> {
        let parser = CommandParser::new();
        parser.parse(line)
    }

    fn execute_parsed_command(&mut self, cmd: ParsedCommand) -> ShellResult<i32> {
        match cmd {
            ParsedCommand::Simple(simple_cmd) => {
                self.execute_simple_command(simple_cmd)
            }
            ParsedCommand::Pipeline(pipeline) => {
                self.execute_pipeline(pipeline)
            }
            ParsedCommand::Conditional(conditional) => {
                self.execute_conditional(conditional)
            }
            ParsedCommand::Background(bg_cmd) => {
                self.execute_background_command(bg_cmd)
            }
        }
    }

    fn execute_simple_command(&mut self, cmd: SimpleCommand) -> ShellResult<i32> {
        let command_name = &cmd.program;
        
        // Check for aliases first
        let actual_command = if self.config.aliases_enabled {
            if let Some(alias) = self.aliases.get(command_name) {
                // Parse the alias and prepend to arguments
                let alias_parts: Vec<&str> = alias.split_whitespace().collect();
                if !alias_parts.is_empty() {
                    let mut new_args = alias_parts[1..].iter().map(|s| s.to_string()).collect::<Vec<_>>();
                    new_args.extend(cmd.args.clone());
                    SimpleCommand {
                        program: alias_parts[0].to_string(),
                        args: new_args,
                        redirections: cmd.redirections,
                        environment: cmd.environment,
                    }
                } else {
                    cmd
                }
            } else {
                cmd
            }
        } else {
            cmd
        };

        // Check for built-in commands
        if let Some(builtin) = get_builtin(&actual_command.program) {
            return builtin.execute(self, &actual_command.args);
        }

        // Check for external commands
        if let Some(external) = get_external_command(&actual_command.program) {
            return external.execute(self, &actual_command);
        }

        // Auto-cd feature
        if self.config.auto_cd && actual_command.args.is_empty() {
            if crate::filesystem::path_exists(&actual_command.program) {
                return self.change_directory(&actual_command.program);
            }
        }

        Err(ShellError::CommandNotFound(actual_command.program))
    }

    fn execute_pipeline(&mut self, pipeline: Pipeline) -> ShellResult<i32> {
        // Implement pipeline execution
        // This would involve creating pipes and connecting commands
        println!("Pipeline execution not yet implemented");
        Ok(0)
    }

    fn execute_conditional(&mut self, conditional: ConditionalCommand) -> ShellResult<i32> {
        // Implement conditional execution (&&, ||)
        println!("Conditional execution not yet implemented");
        Ok(0)
    }

    fn execute_background_command(&mut self, cmd: BackgroundCommand) -> ShellResult<i32> {
        // Implement background job execution
        println!("Background execution not yet implemented");
        Ok(0)
    }

    pub fn change_directory(&mut self, path: &str) -> ShellResult<i32> {
        let new_path = if path.starts_with('/') {
            path.to_string()
        } else if path == ".." {
            // Go up one directory
            let parts: Vec<&str> = self.current_dir.split('/').collect();
            if parts.len() > 1 {
                parts[..parts.len()-1].join("/")
            } else {
                "/".to_string()
            }
        } else if path == "." {
            self.current_dir.clone()
        } else if path == "~" {
            self.variables.get("HOME").unwrap_or("/".to_string())
        } else {
            // Relative path
            if self.current_dir == "/" {
                format!("/{}", path)
            } else {
                format!("{}/{}", self.current_dir, path)
            }
        };

        // Check if directory exists
        if !crate::filesystem::directory_exists(&new_path) {
            return Err(ShellError::DirectoryNotFound(new_path));
        }

        let old_dir = self.current_dir.clone();
        self.current_dir = new_path;
        self.variables.set("PWD".to_string(), self.current_dir.clone());
        self.variables.set("OLDPWD".to_string(), old_dir);

        Ok(0)
    }

    pub fn get_current_directory(&self) -> &str {
        &self.current_dir
    }

    pub fn get_variable(&self, name: &str) -> Option<String> {
        self.variables.get(name)
    }

    pub fn set_variable(&mut self, name: String, value: String) {
        self.variables.set(name, value);
    }

    pub fn get_exit_code(&self) -> i32 {
        self.exit_code
    }

    pub fn exit(&mut self, code: i32) {
        self.exit_code = code;
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}

/// Initialize shell subsystem
pub fn init() {
    println!("Initializing shell subsystem...");
    
    // Initialize command registry
    commands::init_commands();
    builtins::init_builtins();
    
    println!("Shell subsystem initialized");
}

/// Start interactive shell
pub fn start_shell() {
    let mut shell = Shell::new();
    shell.run();
}