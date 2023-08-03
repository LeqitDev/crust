# Crust - A Rust Project Management Tool

Crust is a simple command-line tool written in Rust that helps you manage your Rust projects effectively. With Crust, you can easily create, list, and open Rust projects in different locations on your system. It provides a convenient way to organize your projects and work with them seamlessly.

## General Information

### Features
- Create and manage multiple Rust projects across different locations.
- Open projects in Visual Studio Code for easy editing.
- List all existing projects for quick reference.
- Interactive command-line interface with tab-completion and history support.

### Dependencies
- Rust language: Crust itself is written in Rust, so you'll need to have Rust installed on your system to build and use it.
- Visual Studio Code (optional): Crust can open Rust projects in Visual Studio Code. If you want to use this feature, ensure you have VS Code installed on your machine.

### Usage

1. Download the source code of Crust from [GitHub repository](https://github.com/LeqitDev/crust) and navigate to the project directory.

2. Build the Crust executable:
   ```shell
   cargo build --release
   ```

3. Run Crust:
   ```shell
   ./target/release/crust
   ```

## Commands

1. `help`: Display a help message with a list of available commands and their descriptions.

2. `exit`: Terminate the application and exit.

3. `add-location [path] [prefix]`: Add a new location (parent folder) with a project folder inside. The prefix is used to differentiate between projects with the same name in different locations. The prefix must be unique.

4. `list`: List all existing projects along with their locations.

5. `[prefix].[projectname]`: Open a specific project with Visual Studio Code. If the project name is not recognized, Crust will create a new plain Rust project in the folder associated with the given prefix.

## Getting Started

1. Launch Crust by executing the `./target/release/crust` command in your terminal.

2. You will be greeted with a welcome message and a list of existing projects.

3. Use the available commands to manage your Rust projects. For example:
   - Use `help` to get information about the available commands.
   - Use `list` to see all the existing projects.
   - Use `add-location` to add a new location with a project folder inside.
   - Use `[prefix].[projectname]` to open an existing project with Visual Studio Code.

## Note

- To use the Visual Studio Code integration, ensure that Visual Studio Code is installed and available in your system's PATH.

- The command-line interface supports tab-completion and history. You can use the up and down arrow keys to navigate the command history.

## Contributing

If you find any issues or have suggestions for improving Crust, please feel free to contribute by submitting a pull request on the [GitHub repository](https://github.com/LeqitDev/crust). Your contributions are highly appreciated!

## License

Crust is open-source software licensed under the [MIT License](https://opensource.org/licenses/MIT).

---

Now you have a README file that introduces and explains the Crust Rust project. Feel free to modify and enhance the content as needed. Happy coding with Crust! 🦀
