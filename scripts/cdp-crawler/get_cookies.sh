#!/usr/bin/env bash

set -eo pipefail

if [ "$#" -ne 3 ]; then
    echo "Usage: $0 <cdp_link> username password"
    echo "Example: $0 https://cahier-de-prepa.fr/mp-parc user pass"
    exit 1
fi

link="$1"
username="$2"
password="$3"

# sanitize @ into %40 for curl, and = into %3D
username=$(echo "$username" | sed 's/@/%40/g; s/=/\%3D/g')
password=$(echo "$password" | sed 's/@/%40/g; s/=/\%3D/g')

payload="login=$username&motdepasse=$password&permconn=1&connexion=1"

res=$(
    curl "$link/ajax.php" \
        -H 'Accept: application/json, text/javascript, */*; q=0.01' \
        -H 'Accept-Language: en-US,en;q=0.9,fr;q=0.8' \
        -H 'Connection: keep-alive' \
        -H 'Content-Type: application/x-www-form-urlencoded; charset=UTF-8' \
        -H 'Origin: https://cahier-de-prepa.fr' \
        -H "Referer: $link/" \
        -H 'Sec-Fetch-Dest: empty' \
        -H 'Sec-Fetch-Mode: cors' \
        -H 'Sec-Fetch-Site: same-origin' \
        -H 'User-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36' \
        -H 'X-Requested-With: XMLHttpRequest' \
        -H 'sec-ch-ua: "Not:A-Brand";v="99", "Google Chrome";v="145", "Chromium";v="145"' \
        -H 'sec-ch-ua-mobile: ?0' \
        -H 'sec-ch-ua-platform: "Linux"' \
        --data-raw "$payload" \
        --show-headers
)
echo "Response:
$res
---"

echo "$res" | grep -q "Connexion réussie" || {
    echo "Login failed"
    exit 1
}

cookies=$(echo "$res" | grep -oP 'Set-Cookie: \K[^;]+')
# remove duplicate lines cookie names
cookies=$(echo "$cookies" | awk -F= '!seen[$1]++')

# remove newlines and join with semicolons
cookies=$(echo "$cookies" | tr '\n' '; ' | sed 's/;$//')

echo "Pass the following as the second argument to 'uv run main.py' to use the cookies for authenticated requests:"
echo "$cookies"
