# There is no Container Apps support yet, because of https://github.com/Azure/azure-rest-api-specs/issues/19285
resource "azapi_resource" "managed_environment" {
  type                      = "Microsoft.App/managedEnvironments@2022-03-01"
  name                      = "containerapps-demo"
  parent_id                 = azurerm_resource_group.rg.id
  location                  = azurerm_resource_group.rg.location
  schema_validation_enabled = false

  body = jsonencode({
    properties = {
      internalLoadBalancerEnabled = false
      appLogsConfiguration = {
        destination = "log-analytics"
        logAnalyticsConfiguration = {
          customerId = azurerm_log_analytics_workspace.logs.workspace_id
          sharedKey  = azurerm_log_analytics_workspace.logs.primary_shared_key
        }
      }
    }
  })
}

resource "azapi_resource" "app" {
  type                      = "Microsoft.App/containerapps@2022-03-01"
  name                      = "aztail-containerapp"
  parent_id                 = azurerm_resource_group.rg.id
  location                  = azurerm_resource_group.rg.location
  schema_validation_enabled = false

  body = jsonencode({
    properties = {
      managedEnvironmentId = azapi_resource.managed_environment.id
      template = {
        containers = [
          {
            name  = "log-container"
            image = "ubuntu:latest"
            args = ["/bin/bash", "-c", "while true ; do echo Stdout ; echo Stderr > /dev/stderr ; sleep 60 ; done"]
            resources = {
              cpu    = 0.25
              memory = "0.5Gi"
            }
          }
        ]
        scale = {
          minReplicas = 1
          maxReplicas = 1
        }
      }
    }
  })
}
