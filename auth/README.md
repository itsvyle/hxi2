# auth

Central authentication server for the entire system. This server is responsible for authenticating users and providing them with a token that can be used to access other services.

Uses discord Oauth2 for authentication.

You will need to allow HXI2_AUTH_URL/api/discord_callback as a redirect url in the discord developer portal.

## Run

Required variables:

| Variable                      | Description                                                                                                                        | Example (_not default_) |
| ----------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- | ----------------------- |
| HXI2_AUTH_URL                 | Domain name and protocol of the **internet facing** auth domain (used for redirecting in case the user isn't logged in, to log in) | https://auth.hxi2.com   |
| HXI2_AUTH_ENDPOINT            | Endpoint to call internally to renew tokens, or control other authentication stuff; it can be a local url                          | -                       |
| HXI2_COOKIES_DOMAIN           | Domain of the global cookies, most importantly token/refreshtoken/smalldata                                                        | -                       |
| HXI2_TLD                      | Domain name                                                                                                                        | hxi2.fr                 |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------                       | -                       |
| CONFIG_DEFAULT_REDIRECT_URL   | Default redirect url for the login page after the user has logged in                                                               | -                       |
| CONFIG_RUNNING_PORT           | Port on which the server will run                                                                                                  | -                       |
| CONFIG_JWT_PRIVATE_KEY        | Private key for the JWT token; set it to "generate" to create a random key, which will be outputed to a file                       | -                       |
| CONFIG_DB_PATH                | Path to the sqlite database file                                                                                                   | -                       |
| CONFIG_DISCORD_APPLICATION_ID | Discord application id                                                                                                             | -                       |
| CONFIG_DISCORD_CLIENT_ID      | Discord client id                                                                                                                  | -                       |
| CONFIG_DISCORD_CLIENT_SECRET  | Discord client secret                                                                                                              | -                       |
