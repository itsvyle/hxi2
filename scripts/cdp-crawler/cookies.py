# CDPLINK = "https://cahier-de-prepa.fr/mp2i-parc"
# à obtenir en se connectant à cdp et en regardant les cookies dans le navigateur
# COOKIES = "CDP_SESSION_PERM=xxxxxxxx; CDP_SESSION=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"

SECTIONS = ["phys","general","info","SI","Francais"]

# arguments: link username password sections(comma separated)
import sys
import urllib.error
import urllib.request
import urllib.parse
import http.cookiejar

if len(sys.argv) != 4:
    print("Usage: uv run main.py <link> <cookies> <sections (comma separated)>")
    print("Example: uv run main.py https://cahier-de-prepa.fr/mp2i-parc \"CDP_SESSION_PERM=xxxxxxxx; CDP_SESSION=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\" phys,general,info,SI,Francais")
    print("Get the cookies with ./get_cookies <link> <username> <password>")
    sys.exit(1)

CDPLINK = sys.argv[1]
COOKIES = sys.argv[2]
SECTIONS = sys.argv[3].split(",")