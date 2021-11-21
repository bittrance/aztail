# aztail - cli to retrieve Azure logs

aztail supports App Insights and Log Analytics. Its purpose is to extract logs and to allow tailing logs, with an eye to assisting development and operation of Azure Functions. It is inspired by [awslogs](https://github.com/jorgebastida/awslogs).

**Current status**: aztail is early-phase software: it works, but is feature poor.

## Authentication

aztail does not itself handle authentication, but expect you to have used e.g. `az login` and `az account set` to provide it with a session.

## Install

aztail is a single binary and can be downloaded from the repository [release page](https://github.com/bittrance/aztail/releases). aztail currently supports MacOS X, Linux and Windows x86-64.

## Usage

```
Query the "traces" table in a App Insights or Log Analytics workspace and presents the log entries

USAGE:
    aztail [OPTIONS] --app-id <APP_ID>

OPTIONS:
    -a, --app <APP>                  Show only logs for a specific app
        --app-id <APP_ID>            The UUID of the App Insight or Log Analytics workspace where
                                     logs reside
    -e, --end-time <END_TIME>        Retrieve logs older than this. Can be RFC3339 or informal such
                                     as "30min ago"
    -f, --follow                     Tail a log query. Incompatible with --end-time
        --format <FORMAT>            One of text, json [default: text]
    -h, --help                       Print help information
    -o, --operation <OPERATION>      Show only logs for a specific function
    -s, --start-time <START_TIME>    Retrieve logs newer than this. Can be RFC3339 or informal such
                                     as "yesterday"
    -V, --version                    Print version information
```
