terraform {
  required_providers {
    azapi = {
      version = "0.3.0"
      source = "Azure/azapi"
    }
  }
}

provider "azurerm" {
  features {}
}

provider "azapi" {}

resource "azurerm_resource_group" "rg" {
  name     = "aztail-examples"
  location = "West Europe"
}

resource "azurerm_log_analytics_workspace" "logs" {
  name                = "aztail"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name
  sku                 = "PerGB2018"
  retention_in_days   = 30
}
