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

resource "azurerm_network_interface_security_group_association" "sgassoc" {
  network_interface_id      = azurerm_network_interface.nic.id
  network_security_group_id = azurerm_network_security_group.sg.id
}

resource "azurerm_public_ip" "publicip" {
  name                = "aztail-vm-publicip"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location
  allocation_method   = "Dynamic"
}

resource "azurerm_network_interface" "nic" {
  name                = "aztail-nic"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name

  ip_configuration {
    name                          = "public"
    public_ip_address_id          = azurerm_public_ip.publicip.id
    private_ip_address_allocation = "Dynamic"
    subnet_id                     = azurerm_subnet.snet.id
  }
}

resource "azurerm_linux_virtual_machine" "vm" {
  name                = "aztail-vm"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location
  size                = "Standard_F2"
  admin_username      = "aztail"
  network_interface_ids = [
    azurerm_network_interface.nic.id,
  ]

  admin_ssh_key {
    username   = "aztail"
    public_key = file("~/.ssh/id_rsa.pub")
  }

  os_disk {
    name                 = "aztail-vm_os-disk"
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
  virtual_machine_id   = azurerm_linux_virtual_machine.vm.id
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
  virtual_machine_id   = azurerm_linux_virtual_machine.vm.id
  publisher            = "Microsoft.Azure.Extensions"
  type                 = "CustomScript"
  type_handler_version = "2.0"

  settings = <<EOF
    {"commandToExecute": "echo '*.* @127.0.0.1:25224' > /etc/rsyslog.d/95-omsagent.conf && systemctl restart rsyslog.service"}
EOF
}
