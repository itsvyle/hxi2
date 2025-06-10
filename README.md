# HXi2

# Subprojects information

## Environment variables to set

These are variables to set on basically any program in the ecosystem.

| Variable            | Description                                                                                                                        | Example (_not default_)             |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------- |
| HXI2_AUTH_URL       | Domain name and protocol of the **internet facing** auth domain (used for redirecting in case the user isn't logged in, to log in) | https://auth.hxi2.com               |
| HXI2_TLD            | Domain name                                                                                                                        | hxi2.fr                             |
| HXI2_AUTH_ENDPOINT  | Endpoint to call internally to renew tokens, or control other authentication stuff; it can be a local url or the public one        | https://auth.hxi2.com or auth:42001 |
| HXI2_COOKIES_DOMAIN | Domain of the global cookies, most importantly token/refreshtoken/smalldata - with a dot to make it available domain wide          | .hxi2.fr                            |
| HXI2_PUBLIC_KEY_PEM | The public key used to sign JWTs; entirely **optional**, if not set it will fetch the key from the HXI2_AUTH_ENDPOINT              | -                                   |
