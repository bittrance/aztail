# aztail - cli to retrieve Azure logs

aztail supports App Insights and Log Analytics. Its purpose is to extract logs and to allow tailing logs, with an eye to assisting development and operation of Azure Functions. It is inspired by [awslogs](https://github.com/jorgebastida/awslogs).

**Current status**: aztail is early-phase software: it works, but is feature poor.

## Authentication

aztail does not itself handle authentication, but expect you to have used e.g. `az login` and `az account set` to provide it with a session.

## Usage

```
aztail queries the "traces" table in a App Insights or Log Analytics workspace and presents the log entries

USAGE:
    aztail [OPTIONS] --app-id <APP_ID>

OPTIONS:
        --app-id <APP_ID>            The UUID of the App Insight or Log Analytics workspace where
                                     logs reside
    -e, --end-time <END_TIME>        Retrieve logs older than this. Can be RFC3339 or informal such
                                     as "30min ago"
    -f, --follow                     Tail a log query. Incompatible with --end-time
    -h, --help                       Print help information
    -s, --start-time <START_TIME>    Retrieve logs newer than this. Can be RFC3339 or informal such
                                     as "yesterday"
    -V, --version                    Print version information
```
