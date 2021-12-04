resource "azurerm_container_group" "container" {
  name                = "aztail-container"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name
  os_type             = "Linux"
  restart_policy      = "OnFailure"

  container {
    name     = "log-container"
    image    = "ubuntu:latest"
    commands = ["/bin/bash", "-c", "while true ; do echo Stdout ; echo Stderr > /dev/stderr ; sleep 60 ; done"]
    cpu      = "0.5"
    memory   = "1"

    ports {
      port     = 443
      protocol = "TCP"
    }
  }

  diagnostics {
    log_analytics {
      workspace_id  = azurerm_log_analytics_workspace.logs.workspace_id
      workspace_key = azurerm_log_analytics_workspace.logs.primary_shared_key
    }
  }
}
