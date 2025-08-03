# Developement

Ce dépot centralise les multiples projets qui vivent sous l'écosystème et le site interne hxi2.fr.

Chaque sous-projet doit comporter un fichier `DEV.md`, qui indique les instructions de développement pour ce projet: s'y référer pour les détails de chaque projet.

## Dépendances

La grande dépendance de tous les projets sera d'avoir [just](https://github.com/casey/just), un nouvel équivalent à `make`, qui permet de faire des scripts de construction et d'installation.

Chaque projet comprend une commande `just init`, qui a comme role d'initialiser le projet, en installant les dépendances nécessaires, et en vérifiant que les outils nécessaires sont installés.

## Architecture d'authentification

Grand nombre de pages seront protégées par une authentification, via Discord.

Cette authentification est gérée par un service central d'authentification, sur https://auth.hxi2.fr

Ce service d'authentification délivre aux utilisateurs authentifiés un jeton JWT, qui est signé par une clé privée, et mis dans un cookie global du site; la clé publique correspondante est disponible sur le service d'authentification, et peut être récupérée par les applications clientes, à condition qu'elles diposent d'une clé d'API (s'adresser à un admin pour en obtenir une).

Périodiquement, le jeton JWT expirera, et l'application cliente devra appeler le service d'authentification pour obtenir un nouveau jeton, en utilisant le jeton de rafraîchissement, aussi stocké dans un cookie global du site.

Chaque utilisateur peut avoir plusieurs permissions, qui sont stockées en tant que bitfield, dans le JWT.

Dans la pratique, des bibliothèques sont disponibles pour gérer l'authentification des utilisateurs, et la vérification des permissions, dans les applications clientes.

Afin qu'une application cliente puisse vérifier les permissions, rafraichir des tokens, etc, elle aura besoin d'une clé d'API, qui est délivrée par l'administrateur du service d'authentification (itsvyle)

## Environment variables to set

These are variables to set for basically any program in the ecosystem, which will pull from them at runtime

| Variable            | Description                                                                                                                        | Example (_not default_)             |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------- |
| HXI2_AUTH_URL       | Domain name and protocol of the **internet facing** auth domain (used for redirecting in case the user isn't logged in, to log in) | https://auth.hxi2.com               |
| HXI2_TLD            | Domain name                                                                                                                        | hxi2.fr                             |
| HXI2_AUTH_ENDPOINT  | Endpoint to call internally to renew tokens, or control other authentication stuff; it can be a local url or the public one        | https://auth.hxi2.com or auth:42001 |
| HXI2_COOKIES_DOMAIN | Domain of the global cookies, most importantly token/refreshtoken/smalldata - with a dot to make it available domain wide          | .hxi2.fr                            |
| HXI2_PUBLIC_KEY_PEM | The public key used to sign JWTs; entirely **optional**, if not set it will fetch the key from the HXI2_AUTH_ENDPOINT              | -                                   |
