# Abhyas

Abhyas is a Rust command-line application for managing and interacting with a database of links.

### Features

- **Check Status**: Get the total, completed, and skipped links count.
- **Get Link**: Get random link from the database.
- **Add Link**: Add new links to the database.
- **Search Link**: Search link from the database.
- **Other**: View and interact with other available options.
- **Insert Links from File**: Add links from a specified file to the database.

### Requirements

- [SQLite](https://www.sqlite.org/index.html)

### Installation

1. Clone and build the project:
   ```sh
   git clone https://github.com/halshar/abhyas.git
   cargo build --release
   ```
2. Install using cargo

   ```sh
   cargo install abhyas
   ```

### Usage

To run the application:

```bash
abhyas
```

For inserting links from a file:

```bash
abhyas --file <file_path>
# Replace `<file_path>` with the path to your file.
```

### Usage Notes

- When running the application, follow the on-screen instructions to navigate and interact with the available options.
- Use the `--file` flag to insert links from a specified file.

### Acknowledgments

This project utilizes the `rusqlite` and other Rust libraries.

### Contributing

Contributions are welcome! If you have any suggestions or improvements, please open an issue or create a pull request.

### License

This project is licensed under the [GPLv3 License](./LICENSE).
