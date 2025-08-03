# Pages: Dev documentation

## Définition

Le dossier `HXI2_REPO/pages` est l'endroit où sont tapées les "pages" du site internet. Par "page", je veux dire une page du site internet où l'on aura du texte, des liens, etc, c'est à dire pas de fonctionnalité programmée spécifique à chaque page.\
C'est par exemple la page d'accueil, les pages de ressources, etc..

C'est ici que sont stockées toutes les telles pages du site, peut importe par quel autre projet elle sont utilisées.

## Principe

Les pages sont tapées en _markdown_ (fichiers `.md`).

Elles sont ensuite compilées par `Hugo`, puis `webpack` passe et optimise/bundle le résultat.\
Étant donné que les fichiers HTML des pages sont générés automatiquement, il ne faudra jamais éditer un fichier HTML directement, il se verrait modifié à la prochaine compilation des fichiers markdown

Certaines pages sont privées, ne sont pas dans ce repo ci - pour installer le repo privé, faites `git submodule update --init hxi2-private-pages` dans le dossier `pages` - si vous n'y avez pas accès, merci de contacter @itsvyle pour obtenir un accès.

Chaque page dispose d'un fichier typescrip et d'un fichier scss associés, afin de rajouter du scriting ou du styling spécifique

## Nécessités de développement

> [!NOTE]
> Pour l'instant, le développement n'a été testé que sur Linux - pour ce projet spécifique (`pages`), je suis pret à faire des efforts pour supporter Windows, donc en cas de problèmes, contacter @itsvyle

Afin de contribuer aux pages, vous aurez besoin:

- d'avoir la CLI `just` installé sur votre système (https://github.com/casey/just)
- d'avoir la CLI `hugo` installée sur votre système (https://gohugo.io/installation/)
- d'avoir la CLI `volta` installée sur votre système (https://docs.volta.sh/guide/getting-started)
  - Alternativement, vous pouvez avoir juste `node` et `pnpm`

**Une fois les outils installés**, il vous faudra éxecuter au moins une fois la commande `just init` dans le dossier `HXI2_REPO/pages`

## Développement

### Serveur de preview

Durant l'écriture des pages, il sera pratique d'avoir, au fur et à mesure, un preview de la page finalisée: cela est possible via un serveur local de développement.

Pour le démarrer, il suffit d'éxécuter `just dev`: un navigateur s'ouvrira au bon URL, et les pages se rafraichiront d'elles meme lorsqu'elles sont modifiées.

Le serveur local de développement devra etre redémarré si on ajoute un nouveau fichier, ou que l'on change du code des templates hugo directement.

### Créer une nouvelle page

> [!NOTE]
> Pour l'instant, il n'est possible de créer des nouvelles pages de cette manière que sur Linux et MacOS

1. Naviguez vers le dossier dans lequel vous voulez créer une nouvelle page
2. Éxécutez `just create-page <le nom de votre nouvelle page>`
3. Si vous aviez le serveur de développement actif, le redémarrer.

```bash
# Exemple
❯ cd public-pages
❯ just create-page test
Creating test.ts, test.scss, and test.md in section public-pages
Updating sections.json
```

**Remarque**: si vous avez créé un nouveau dossier qui est directement fils de `HXI2_REPO/pages`, vous devrez l'ajouter en module mount à la fin de `HXI2_REPO/pages/hugo.toml`

## Production

Pour build: `just frontend-build`

> [!TIP]
> Il est possible de ne build qu'une seule section, en faisant `PAGES_SECTION="section" just frontend-build`
