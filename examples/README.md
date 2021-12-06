# Logging aggregation from Azure services

This page document how the standard aztail concepts are mapped to tables and columns in Azure Monitoring.

## Legacy Application Insights

| Datum     | Field          |
| --------- | -------------- |
| Table     | traces         |
| Timestamp | timestamp      |
| Group     | operation_Name |
| Unit      | cloud_RoleName |
| Message   | message        |

## Function on Log Analytics

| Datum     | Field         |
| --------- | ------------- |
| Table     | AppTraces     |
| Timestamp | TimeGenerated |
| Group     | operationName |
| Unit      | AppRoleName   |
| Message   | Message       |

## Container instances

| Datum     | Field                   |
| --------- | ----------------------- |
| Table     | ContainerInstanceLog_CL |
| Timestamp | TimeGenerated           |
| Group     | ContainerGroup_s        |
| Unit      | ContainerName_s         |
| Message   | Message                 |

Resources:

- [Container group and instance logging with Azure Monitor logs](https://docs.microsoft.com/en-us/azure/container-instances/container-instances-log-analytics)

## API Management on Application Insights

| Datum     | Field                                                                 |
| --------- | --------------------------------------------------------------------- |
| Table     | requests                                                              |
| Timestamp | timestamp                                                             |
| Group     | customDimensions."API Name"                                           |
| Unit      | customDimensions."Operation Name"                                     |
| Message   | Apache-style with url, resultCode, customMeasurements."Response Size" |

| Datum     | Field                            |
| --------- | -------------------------------- |
| Table     | exceptions                       |
| Timestamp | timestamp                        |
| Group     | cloud_RoleName starts with "{}." |
| Unit      | -                                |
| Message   | outerMessage + operation_Name    |

## Logic App

| Datum     | Field                    |
| --------- | ------------------------ |
| Table     | AzureDiagnostics         |
| Timestamp | TimeGenerated            |
| Group     | resource_workflowName_s  |
| Unit      | Resource                 |
| Message   | OperationName + status_s |

## Azure Linux VM with OMS logging to Syslog

| Datum     | Field                     |
| --------- | ------------------------- |
| Table     | Syslog                    |
| Timestamp | TimeGenerated             |
| Group     | Computer                  |
| Unit      | ProcessName               |
| Message   | SyslogMessage + ProcessID |

Resources:

- https://trstringer.com/systemd-journal-to-syslog-azure-monitoring/
- https://docs.microsoft.com/en-us/azure/virtual-machines/extensions/oms-linux
- https://docs.microsoft.com/en-us/azure/azure-monitor/agents/agents-overview
- https://docs.microsoft.com/en-us/azure/azure-monitor/agents/data-sources-syslog

## Future data points

Instance:

- Function app invocation id
- Container id
