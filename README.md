# LLM Writing Feedback

A real-time writing assistant that provides instant feedback on your writing using Anthropic's Claude AI model.

## Features

- Real-time document monitoring and feedback
- Maintains context of previous versions
- Provides incremental feedback on changes
- Supports Markdown files
- Clean terminal-based interface

## Prerequisites

- Rust
- An Anthropic API key

## Installation

1. Clone the repository
2. Create a `.env` file in the root directory with the following variables:
   ```
   ANTHROPIC_API_KEY=anthropic_api_key
   ```

## Usage

Run the application with a target markdown file:
```
cargo run --release -- path/to/your/file.md
```
