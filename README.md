<div align="center">
  <h3 align="center">Mnemnk Obsidian Agents</h3>

  <p align="center">
    Mnemnk agents for integration with Obsidian via the Local REST API
  </p>

![Badge Language] 
[![Badge License]][License]

</div>

## Getting Started

### Prerequisites

- [Mnemnk App](https://github.com/mnemnk/mnemnk-app)
- [Rust](https://www.rust-lang.org/)
- [Obsidian](https://obsidian.md/)
- [Local REST API](https://github.com/coddingtonbear/obsidian-local-rest-api)

### Installation

1. Clone the repo into the `{Mnemnk Directory}/agents/`

    ```shell
    cd {Mnemnk Directory}/agents/
    git clone https://github.com/mnemnk/mnemnk-obsidian.git
    ```

2. Build

    ```shell
    cd mnemnk-obsidian
    cargo build --release
    ```

<!----------------------------------------------------------------------------->

[License]: https://github.com/mnemnk/mnemnk-obsidian

<!----------------------------------{ Badges }--------------------------------->

[Badge Language]: https://img.shields.io/github/languages/top/mnemnk/mnemnk-obsidian
[Badge License]: https://img.shields.io/badge/License-MIT%20or%20Apache%202-green.svg
