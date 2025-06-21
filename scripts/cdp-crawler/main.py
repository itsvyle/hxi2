from bs4 import BeautifulSoup
from urllib import request as urlr
import os
from cookies import *
CDPLINK = "https://cahier-de-prepa.fr/mp2i-parc"
# à obtenir en se connectant à cdp et en regardant les cookies dans le navigateur
# COOKIES = "CDP_SESSION_PERM=xxxxxxxx; CDP_SESSION=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"


SECTIONS = ["phys","general","info","SI","Francais"]

def rendre_nom_valide(nom):
    """
    retire ^/:*?<>""| (oui, c'est moche)
    """
    return "".join([c for c in nom if not c in "^/:*?<>\"\\|"])

def enleve_espace_fin (nom):
    if nom[-1] == " ":
        return nom[:-1]
    return nom

def extract_folders_files(s : BeautifulSoup) :
    ps = s.body.section.find_all("p")
    foldlinks = []
    filelinks = []
    for p in ps :
        if p["class"] == ['rep'] :
            nom = enleve_espace_fin(rendre_nom_valide(p.find(class_ = "nom").string)) #si on rend le nom valide alors il if finiras sûrment par un espace, ce que le module os ne vérifie pas et ignore et dcp les chemins sont faux
            foldlinks.append((p.a.get("href"), nom))
        elif p["class"] == ['doc'] and "/" not in p.find(class_ = "nom").string : # le cas '/' se présente sur la première page à cause de la section "documents récents"
            fext = p.find(class_ = "docdonnees").string[1:-1]
            fext = fext.split(",")[0]
            nom = enleve_espace_fin(rendre_nom_valide(p.find(class_ = "nom").string+'.'+fext))
            filelinks.append((p.a.get("href"), nom))

    return foldlinks, filelinks

def rec_extract_folders_files(url, root = ''):
    """
    Note : ici "url" est une chaîne du type "?orga" qui suivera un "/docs" 
    """
    folders = []
    files = []
    
    def aux(fp, url) : # en parcours "postfixe" pour que recréer les dossiers soit possible en dépilant
        req = urlr.Request(
            f"{CDPLINK}/docs{url}",
            headers = {
                "Cookie": COOKIES
            }
        )

        with urlr.urlopen(req) as f :
            soup = BeautifulSoup(f.read(), features="html.parser")
        
        foldlinks, filelinks = extract_folders_files(soup)
        files.extend([(fp+"/"+f[1], f[0]) for f in filelinks])
        for f in foldlinks :
            aux(fp+"/"+f[1], f[0])
            folders.append(fp+"/"+f[1])
            
    aux(root, url)
    return folders, files

def download_and_save(fp, url):
    req = urlr.Request(
            f"{CDPLINK}/{url}",
            headers = {
                "Cookie": COOKIES
            }
        )
    
    with urlr.urlopen(req) as f :
        data = f.read()
    
    with open(fp,'wb') as f:
        f.write(data)

def create_folders_files(url, root = ".") :
    print("Extraction de la page...")
    folds, files = rec_extract_folders_files(url, root)
    folds.append(root)
    
    print("Création des dossiers...")
    for fp in folds :
        if not os.path.exists(fp):
            os.makedirs(fp)
    
    print("Téléchargement des fichiers :")
    for f in files :
        print(f"    - Téléchargement de {f[0]}...")
        if not os.path.exists(f[0]):
            download_and_save(*f)

    print("Voilà, c'est terminé gros.")


def main():
    for tipe in SECTIONS :
        print("./output/"+tipe)
        if not os.path.exists("./output/"+tipe):
                os.makedirs("./output/"+tipe)
        create_folders_files("?"+tipe, root = "./output/"+tipe)


if __name__ == "__main__":
    main()
