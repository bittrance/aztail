use serde_json::{json, Map, Value};

pub const T1: &str = "2021-11-20T06:18:30+00:00";
pub const T2: &str = "2021-11-20T06:18:31+00:00";
pub const T3: &str = "2021-11-20T06:18:32+00:00";
pub const T4: &str = "2021-11-20T06:18:33+00:00";

pub fn requests_http_row() -> Map<String, Value> {
    json!({
        "client_Browser": "",
        "client_IP": "83.248.129.91",
        "client_Model": "",
        "client_OS": "",
        "client_Type": "PC",
        "cloud_RoleName": "aztail-apim.azure-api.net West Europe",
        "customDimensions": "{\"API Type\":\"http\",\"Subscription Name\":\"master\",\"Operation Name\":\"get-ping\",\"Region\":\"West Europe\",\"API Revision\":\"1\",\"Request Id\":\"e84f10ca-b9f8-40ab-8ed8-6e3588445262\",\"Service Name\":\"aztail-apim.azure-api.net\",\"Request-accept\":\"*/*\",\"Cache\":\"None\",\"Service Type\":\"API Management\",\"Response-content-length\":\"0\",\"API Name\":\"aztail-api\",\"HTTP Method\":\"GET\"}",
        "customMeasurements": "{\"Response Size\":93,\"Request Size\":0,\"Client Time (in ms)\":0}",
        "duration": 0.2486,
        "itemCount": 1,
        "name": "GET /example/",
        "operation_Name": "aztail-api;rev=1 - get-ping",
        "resultCode": "200",
        "session_Id": "",
        "source": "",
        "success": "True",
        "timestamp": "2021-12-22T22:56:48.164Z",
        "url": "https://aztail-apim.azure-api.net/example/?foo=bar",
        "user_AccountId": "",
        "user_AuthenticatedId": "",
        "user_Id": ""
    }).as_object().unwrap().clone()
}

pub fn apprequests_functions_row() -> Map<String, Value> {
    json!({
        "AppRoleName": "aztail-function",
        "ClientIP": "0.0.0.0",
        "ClientModel": "",
        "ClientOS": "",
        "DurationMs": 6.4568,
        "Measurements": null,
        "Name": "log-function",
        "OperationName": "log-function",
        "Properties": "{\"FunctionExecutionTimeMs\":\"5.6831\",\"InvocationId\":\"82e23ea3-9264-427c-b87d-d1816229919c\",\"HostInstanceId\":\"d36a0d40-49d7-498a-9476-8e638693e3f1\",\"ProcessId\":\"70\",\"TriggerReason\":\"Timer fired at 2021-12-30T23:16:00.0002367+00:00\",\"Category\":\"Host.Results\",\"FullName\":\"Functions.log-function\",\"LogLevel\":\"Information\",\"OperationName\":\"log-function\"}",
        "ResultCode": "0",
        "SessionId": "",
        "Source": "",
        "Success": true,
        "TimeGenerated": "2021-12-30T23:16:00Z",
        "Url": "",
        "UserAccountId": "",
        "UserAuthenticatedId": "",
        "UserId": "",
    }).as_object().unwrap().clone()
}

pub fn traces_functions_row() -> Map<String, Value> {
    json!({
        "timestamp": T1,
        "cloud_RoleName": "ze-app",
        "operation_Name": "ze-operation",
        "message": "ze-message",
        "severityLevel": 1,
    })
    .as_object()
    .unwrap()
    .clone()
}

pub fn container_apps_row() -> Map<String, Value> {
    json!({
        "Computer": "",
        "ContainerAppName_s": "aztail-containerapp",
        "ContainerGroupId_g": "26d54061-b088-4ad7-a8ca-bae0990785e4",
        "ContainerGroupName_s": "aztail-containerapp--50rufil-6f767b7c8d-wjnh9",
        "ContainerId_s": "c7d965c2003bfaac29d132a85ae7d7b23ca2794e2173e4c10e9522c4414a5d12",
        "ContainerImage_s": "ubuntu:latest",
        "ContainerName_s": "log-container",
        "EnvironmentName_s": "proudbay-f42875ce",
        "Log_s": "Stdout",
        "MG": "",
        "ManagementGroupName": "",
        "RawData": "",
        "RevisionName_s": "aztail-containerapp--50rufil",
        "SourceSystem": "RestAPI",
        "Stream_s": "stdout",
        "TenantId": "a9d042e7-d348-4f1e-a5b2-fb817144f9d6",
        "TimeGenerated": "2022-09-25T20:35:43.597Z",
        "Type": "ContainerAppConsoleLogs_CL",
        "_ResourceId": "",
        "_timestamp_d": 1664138142.74275,
        "logtag_s": "F",
        "time_s": "",
        "time_t": "2022-09-25T20:35:42.742Z",
    })
    .as_object()
    .unwrap()
    .clone()
}
