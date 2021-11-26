# Balder integrations on Azure Functions

## Included integrations

### pm-properties-to-d365-crm

Updates properties in Balder's Dynamics CRM instance with data from PM (as exposed by the "Yggdrasil" API gateway).

The integration is configured using environment variables. The following variables are recognized:

| Environment variable   | Description                                                                            |
| ---------------------- | -------------------------------------------------------------------------------------- |
| AD_TENANT_ID           | Azure AD tenant from which to request access token for Dynamics CRM access             |
| DYNAMICS_CLIENT_ID     | Service principal client ID with permissions to access Dynamics CRM                    |
| DYNAMICS_CLIENT_SECRET | Service principal secret to access Dynamics CRM                                        |
| DYNAMICS_INSTANCE      | URL prefix for the Dynamics CRM instance to push data to                               |
| PM_URL_PREFIX          | URL prefix for the "Yggdrasil" API gateway where /upm-employee/Properties is published |
| PM_SUBSCRIPTION_KEY    | API gateway subscription key with permission to use the relevant APIs                  |
| DRY_RUN                | When set, no updates are pushed to Dynamics CRM                                        |

## Develop

For general guidelines on Node.JS Azure Functions, see [Azure Functions JavaScript developer guide](https://docs.microsoft.com/en-us/azure/azure-functions/functions-reference-node).

These integrations are written in JavaScript for Node.JS v14. They are written using modern [ESM](https://tc39.es/ecma262/#sec-modules) style (i.e. `import foo from "bar"` rather than `const foo = require("bar")`, also known as "CommonJS" style). This allows usage of modern JavaScript features across the board without resorting to Babel. Azure Functions supports this format. However, despite the specification being properly adopted, this support is still marked as experiemental in Node.JS and so if you want to run this code manually, you will need to do `node --experimental-vm-modules`.

Install dependencies:

```shell
npm ci
```

Run the tests (including coverage):

```
npm test
```

If you want to run an integration locally, you can use the provided wrapper. Don't forget to set the relevant environment variables.

```shell
node --experimental-vm-modules ./manual.mjs ./pm-properties-to-d365-crm/index.mjs
```

## Deploy

The integrations are automatically deployed when changes are merged to master, so you do not normally need to deploy manually. Should you want to do a manual deploy, you can run:

```shell
npm install -g azure-functions-core-tools@3
az login
az account set --subscription <subscription-name>
rm -rf node_modules/
npm ci --only=production
func azure functionapp publish <function-name>
```

In order to get back a working dev environment, you need to `npm install` again.
