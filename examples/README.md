# Logging aggregation from Azure services

This page document how the standard aztail concepts are mapped to tables and columns in Azure Monitoring.

## Azure Functions on Application Insights

| Datum     | Field          | Arg            |
| --------- | -------------- | -------------- |
| Table     | traces         |                |
| Timestamp | timestamp      |                |
| Group     | operation_Name | --function-app |
| Unit      | cloud_RoleName | --function     |
| Level     | severityLevel  |                |
| Message   | message        |                |

## Azure Functions on Log Analytics

| Datum     | Field         | Arg            |
| --------- | ------------- | -------------- |
| Table     | AppTraces     |                |
| Timestamp | TimeGenerated |                |
| Group     | AppRoleName   | --function-app |
| Unit      | OperationName | --function     |
| Level     | severityLevel |                |
| Message   | Message       |                |

## Container instances - TBD

| Datum     | Field                   | Arg               |
| --------- | ----------------------- | ----------------- |
| Table     | ContainerInstanceLog_CL |                   |
| Timestamp | TimeGenerated           |                   |
| Group     | ContainerGroup_s        | --container-group |
| Unit      | ContainerName_s         | --container-name  |
| Message   | Message                 |                   |

Resources:

- [Container group and instance logging with Azure Monitor logs](https://docs.microsoft.com/en-us/azure/container-instances/container-instances-log-analytics)

## API Management on Application Insights

| Datum     | Field                            | Arg             |
| --------- | -------------------------------- | --------------- |
| Table     | requests                         |                 |
| Timestamp | timestamp                        |                 |
| Group     | cloud_RoleName starts with "{}." | --api-name      |
| Unit      | operation_Name ends with " {}"   | --api-operation |
| Level     | success info/warn                |                 |
| Message   | Apache-style [1]                 |                 |

- [1] with url, resultCode, customMeasurements."Response Size"
- Note: It would also be possible to use OperationName ("aztail-api;rev=1 - get-ping") with startswith/endswith which would allow filtering on revision as well.

| Datum     | Field                            | Arg                 |
| --------- | -------------------------------- | ------------------- |
| Table     | exceptions                       |                     |
| Timestamp | timestamp                        |                     |
| Group     | cloud_RoleName starts with "{}." | --api-name          |
| Unit      | operation_Name ends with " {}"   | --api-operation [1] |
| Message   | outerMessage + operation_Name    |                     |

- [1] We may want to warn that some exceptions may be excluded

## API Management on Log Analytics

| Datum     | Field                         | Arg             |
| --------- | ----------------------------- | --------------- |
| Table     | AppRequests                   |                 |
| Timestamp | TimeGenerated                 |                 |
| Group     | AppRoleName starts with "{}." | --api-name      |
| Unit      | OperationName ends with " {}" | --api-operation |
| Message   | Apache-style [1]              |                 |

- [1] with url, resultCode, customMeasurements."Response Size"

| Datum     | Field                         | Arg                 |
| --------- | ----------------------------- | ------------------- |
| Table     | AppExceptions                 |                     |
| Timestamp | TimeGeneratesd                |                     |
| Group     | AppRoleName starts with "{}." | --api-name          |
| Unit      | OperationName ends with " {}" | --api-operation [1] |
| Message   | outerMessage + operation_Name |                     |

- [1] We may want to warn that some exceptions may be excluded

## Logic App - TBD

| Datum     | Field                    | Arg        |
| --------- | ------------------------ | ---------- |
| Table     | AzureDiagnostics         |            |
| Timestamp | TimeGenerated            |            |
| Group     | resource_workflowName_s  | --workflow |
| Unit      | Resource                 | ?          |
| Message   | OperationName + status_s |            |

## Azure Linux VM with OMS logging to Syslog - TBD

| Datum     | Field                     | Arg        |
| --------- | ------------------------- | ---------- |
| Table     | Syslog                    |            |
| Timestamp | EventTime                 |            |
| Group     | Computer                  | --computer |
| Unit      | ProcessName               | --process  |
| Message   | SyslogMessage + ProcessID |            |

Resources:

- https://trstringer.com/systemd-journal-to-syslog-azure-monitoring/
- https://docs.microsoft.com/en-us/azure/virtual-machines/extensions/oms-linux
- https://docs.microsoft.com/en-us/azure/azure-monitor/agents/agents-overview
- https://docs.microsoft.com/en-us/azure/azure-monitor/agents/data-sources-syslog

## Future data points

Instance:

- Function app invocation id
- Container id

# Notes

Does App insight/Log analytics give any guarantee about preserving ingenstion order?
https://stackoverflow.com/questions/49102487/app-insights-traces-are-out-of-order-azure-functions-app

We currently do strict gt and lt, but actually, the ts from the user should prolly be treated as ge/le while advance_start would want to use gt.
