provider "http" {}

data "http" "local_ip" {
  url = "https://ifconfig.co"
}

resource "azurerm_virtual_network" "vnet" {
  name                = "aztail-network"
  address_space       = ["10.0.0.0/16"]
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name
}

resource "azurerm_subnet" "snet" {
  name                 = "aztail-subnet"
  resource_group_name  = azurerm_resource_group.rg.name
  virtual_network_name = azurerm_virtual_network.vnet.name
  address_prefixes     = ["10.0.2.0/24"]
}

resource "azurerm_network_security_group" "sg" {
  name                = "aztail-vm-limit"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name

  security_rule {
    name                       = "allow-ssh"
    priority                   = 100
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Tcp"
    source_port_range          = "*"
    destination_port_range     = "22"
    source_address_prefix      = "${chomp(data.http.local_ip.body)}/32"
    destination_address_prefix = "*"
  }

  security_rule {
    name                       = "deny-all"
    priority                   = 200
    direction                  = "Inbound"
    access                     = "Deny"
    protocol                   = "*"
    source_port_range          = "*"
    destination_port_range     = "*"
    source_address_prefix      = "*"
    destination_address_prefix = "*"
  }
}

resource "azurerm_network_interface_security_group_association" "sgassoc_oms" {
  network_interface_id      = azurerm_network_interface.nic_oms.id
  network_security_group_id = azurerm_network_security_group.sg.id
}

resource "azurerm_public_ip" "publicip_oms" {
  name                = "aztail-vm-publicip-oms"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location
  allocation_method   = "Dynamic"
}

resource "azurerm_network_interface" "nic_oms" {
  name                = "aztail-nic-oms"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name

  ip_configuration {
    name                          = "public"
    public_ip_address_id          = azurerm_public_ip.publicip_oms.id
    private_ip_address_allocation = "Dynamic"
    subnet_id                     = azurerm_subnet.snet.id
  }
}

resource "azurerm_linux_virtual_machine" "vm_oms" {
  name                = "aztail-vm-oms"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location
  size                = "Standard_F2"
  admin_username      = "aztail"
  network_interface_ids = [
    azurerm_network_interface.nic_oms.id,
  ]

  admin_ssh_key {
    username   = "aztail"
    public_key = file("~/.ssh/id_rsa.pub")
  }

  os_disk {
    name                 = "aztail-vm-oms_os-disk"
    caching              = "ReadWrite"
    storage_account_type = "Standard_LRS"
  }

  source_image_reference {
    publisher = "Canonical"
    offer     = "UbuntuServer"
    sku       = "18.04-LTS"
    version   = "latest"
  }
}

resource "azurerm_virtual_machine_extension" "oms" {
  name                 = "aztail-logs"
  virtual_machine_id   = azurerm_linux_virtual_machine.vm_oms.id
  publisher            = "Microsoft.EnterpriseCloud.Monitoring"
  type                 = "OmsAgentForLinux"
  type_handler_version = "1.13"

  settings           = <<EOF
    {"workspaceId":"${azurerm_log_analytics_workspace.logs.workspace_id}"}
EOF
  protected_settings = <<EOF
    {"workspaceKey":"${azurerm_log_analytics_workspace.logs.primary_shared_key}"}
EOF
}

resource "azurerm_virtual_machine_extension" "rsyslog_config" {
  name                 = "aztail-vm-oms2syslog"
  virtual_machine_id   = azurerm_linux_virtual_machine.vm_oms.id
  publisher            = "Microsoft.Azure.Extensions"
  type                 = "CustomScript"
  type_handler_version = "2.0"

  settings = <<EOF
    {"commandToExecute": "echo '*.* @127.0.0.1:25224' > /etc/rsyslog.d/95-omsagent.conf && systemctl restart rsyslog.service"}
EOF
}

resource "azurerm_network_interface_security_group_association" "sgassoc_ama" {
  network_interface_id      = azurerm_network_interface.nic_ama.id
  network_security_group_id = azurerm_network_security_group.sg.id
}

resource "azurerm_public_ip" "publicip_ama" {
  name                = "aztail-vm-publicip-ama"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location
  allocation_method   = "Dynamic"
}

resource "azurerm_network_interface" "nic_ama" {
  name                = "aztail-nic-ama"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name

  ip_configuration {
    name                          = "public"
    public_ip_address_id          = azurerm_public_ip.publicip_ama.id
    private_ip_address_allocation = "Dynamic"
    subnet_id                     = azurerm_subnet.snet.id
  }
}

resource "azurerm_linux_virtual_machine" "vm_ama" {
  name                = "aztail-vm-ama"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location
  size                = "Standard_F2"
  admin_username      = "aztail"
  network_interface_ids = [
    azurerm_network_interface.nic_ama.id,
  ]

  identity {
    type = "SystemAssigned"
  }

  admin_ssh_key {
    username   = "aztail"
    public_key = file("~/.ssh/id_rsa.pub")
  }

  os_disk {
    name                 = "aztail-vm-ama_os-disk"
    caching              = "ReadWrite"
    storage_account_type = "Standard_LRS"
  }

  source_image_reference {
    publisher = "Canonical"
    offer     = "0001-com-ubuntu-server-focal"
    sku       = "20_04-lts"
    version   = "latest"
  }
}

resource "azurerm_virtual_machine_extension" "ama" {
  name                       = "aztail-vm-ama"
  virtual_machine_id         = azurerm_linux_virtual_machine.vm_ama.id
  publisher                  = "Microsoft.Azure.Monitor"
  type                       = "AzureMonitorLinuxAgent"
  type_handler_version       = "1.25"
  auto_upgrade_minor_version = "true"
  depends_on                 = [azurerm_log_analytics_workspace.logs]
}

resource "azurerm_monitor_data_collection_rule" "dcr" {
  name                = "aztail-dcr"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location

  destinations {
    log_analytics {
      name                  = "aztail-logs"
      workspace_resource_id = azurerm_log_analytics_workspace.logs.id
    }

    azure_monitor_metrics {
      name = "aztail-metrics"
    }
  }

  data_flow {
    streams      = ["Microsoft-InsightsMetrics"]
    destinations = ["aztail-metrics"]
  }

  data_flow {
    streams      = ["Microsoft-InsightsMetrics", "Microsoft-Syslog", "Microsoft-Perf"]
    destinations = ["aztail-logs"]
  }

  data_sources {
    syslog {
      facility_names = ["*"]
      log_levels     = ["*"]
      name           = "aztail-syslog"
    }

    performance_counter {
      streams                       = ["Microsoft-Perf", "Microsoft-InsightsMetrics"]
      sampling_frequency_in_seconds = 60
      counter_specifiers            = ["Processor(*)\\% Processor Time"]
      name                          = "aztail-perfcounter"
    }
  }
}

resource "azurerm_monitor_data_collection_rule_association" "dcrassoc" {
  name                    = "aztail-dcr-assoc"
  target_resource_id      = azurerm_linux_virtual_machine.vm_ama.id
  data_collection_rule_id = azurerm_monitor_data_collection_rule.dcr.id
  description             = "aztail-dcr-assoc"
}
