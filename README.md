# Rust website monitor

Website monitor implemented in Rust using the reqwest package. This can be used to track changes on websites. For example, online stores can be monitored for an item restocking. Whenever a page update is detected, a message is sent on discord.

## Requirements

* Discord webhook.

## Build
Build the application with:
```
cargo build
```

## Usage
* Create `sites.json` in the repository root to configure which pages to monitor. See `sites.example.json` for reference.
* Create `proxies.txt` to configure which proxies to use. Leave the file empty to run from localhost.
    * Each line in `proxies.txt` should define 1 proxy using the following format:
        ```
        host:port:username:password
        ```
    * If the proxy does not require authentication, use the following format:
        ```
        host:port
        ```
* Create `env.json` to configure the discord webhook.
* Run with:
    ```
    cargo run SITE1 SITE2 ...
    ```
* Each site is defined in `sites.json`. For example, a monitor for the playstation 5 is configured in `sites.example.json`. To monitor this page, run:
    ```
    cargo run ps5
    ```

## Development

Existing implementations can be made to target specific pages by editing `sites.json`.

New implementations can be made inside `src/monitor/sites`. A new implementation is made by implementing the `MonitorProcessing` trait. See the included examples for details.

## Examples
Examples include implementations for  nike.com, nvidia.com, shopify, bol.com and marktplaats.nl.
