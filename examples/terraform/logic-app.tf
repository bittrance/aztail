resource "azurerm_logic_app_workflow" "logicapp" {
  name                = "aztail-logicapp"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name
}

resource "azurerm_logic_app_trigger_recurrence" "recurrence" {
  name         = "every-minute"
  logic_app_id = azurerm_logic_app_workflow.logicapp.id
  frequency    = "Minute"
  interval     = 1
}

resource "azurerm_logic_app_action_http" "ifconfig" {
  name         = "ifconfig"
  logic_app_id = azurerm_logic_app_workflow.logicapp.id
  method       = "GET"
  uri          = "https://ifconfig.co"
}
