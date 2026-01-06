# PartiQL Beamline Guide

This is the comprehensive guide for PartiQL Beamline, a tool for fast data generation for PartiQL testing and experimentation.

## Building the Book

This book is built using [mdBook](https://rust-lang.github.io/mdBook/). To build and view the book:

### Prerequisites

Install mdBook:

```bash
cargo install mdbook
```

### Building

From the `partiql-beamline-book` directory:

```bash
# Build the book
mdbook build

# Serve the book locally (with auto-reload)
mdbook serve

# Open in browser
mdbook serve --open
```

The built book will be available in the `book/` directory, and the development server will run at `http://localhost:3000`.

## Book Structure

The book is organized into the following sections:

- **Introduction**: Overview and welcome
- **Getting Started**: Installation and first steps
- **Understanding the Basics**: Core concepts and fundamentals
- **Data Generation**: Comprehensive data generation guide
- **Query Generation**: PartiQL query generation
- **Schema and Shape Inference**: Working with data schemas
- **Database Generation**: Creating complete databases
- **Command Line Interface**: Complete CLI reference
- **Advanced Topics**: Performance, customization, and integration
- **Reference**: Quick lookup for generators, commands, and configuration
- **Examples and Tutorials**: Hands-on tutorials with real-world scenarios

## Contributing

To contribute to the documentation:

1. Edit the Markdown files in the `src/` directory
2. Test your changes with `mdbook serve`
3. Submit a pull request

### Adding New Pages

1. Create a new Markdown file in the appropriate `src/` subdirectory
2. Add the page to `src/SUMMARY.md` in the correct location
3. Build and test the book

### Writing Guidelines

- Use clear, concise language
- Include practical examples and code snippets
- Test all commands and examples
- Use consistent formatting and style
- Include cross-references to related sections

## Book Configuration

The book configuration is in `book.toml`. Key settings:

- **Title**: PartiQL Beamline Guide
- **Authors**: PartiQL Beamline Contributors
- **Source**: `src/` directory
- **Output**: HTML format with GitHub integration

## Deployment

The book can be deployed to:

- GitHub Pages
- Static hosting services
- Internal documentation systems
- Local file systems

For GitHub Pages deployment, the built book in the `book/` directory can be published directly.
