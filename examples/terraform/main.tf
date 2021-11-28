provider "azurerm" {
  features {}
}

resource "azurerm_resource_group" "rg" {
  name     = "aztail-example-function"
  location = "West Europe"
}

resource "azurerm_storage_account" "storage_account" {
  name                     = "aztailfunction"
  resource_group_name      = azurerm_resource_group.rg.name
  location                 = azurerm_resource_group.rg.location
  account_tier             = "Standard"
  account_replication_type = "LRS"
}

resource "azurerm_app_service_plan" "app_service_plan" {
  name                = "plan-aztail-function"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name
  kind                = "FunctionApp"
  reserved            = true

  sku {
    tier = "Dynamic"
    size = "Y1"
  }
}

resource "azurerm_application_insights" "application_insights" {
  name                = "appi-aztail"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location
  application_type    = "Node.JS"
}

resource "azurerm_function_app" "function_app" {
  name                       = "func-aztail-example-function"
  resource_group_name        = azurerm_resource_group.rg.name
  location                   = azurerm_resource_group.rg.location
  version                    = "~3"
  storage_account_name       = azurerm_storage_account.storage_account.name
  storage_account_access_key = azurerm_storage_account.storage_account.primary_access_key
  app_service_plan_id        = azurerm_app_service_plan.app_service_plan.id
  app_settings = {
    "APPINSIGHTS_INSTRUMENTATIONKEY" = azurerm_application_insights.application_insights.instrumentation_key,
    "FUNCTIONS_WORKER_RUNTIME"       = "node",
    "WEBSITE_RUN_FROM_PACKAGE"       = "",
  }

  os_type = "linux"
  site_config {
    linux_fx_version          = "node|14"
    use_32_bit_worker_process = false
  }

  lifecycle {
    ignore_changes = [
      app_settings["WEBSITE_RUN_FROM_PACKAGE"],
    ]
  }
}
