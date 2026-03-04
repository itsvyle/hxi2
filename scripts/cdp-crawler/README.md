# CDP Scrapper

0. Install uv: `curl -LsSf https://astral.sh/uv/install.sh | sh`
1. Find your link; should be `https://cahier-de-prepa.fr/<name here>`
2. Run `./get_cookies <link> <username> <password>`, and the cookies string
3. On the cahier de prépa, click on the categories; the url should be like `https://cahier-de-prepa.fr/<name>/docs?maths`, meaning this category name is `maths`; do this for all the sections, and craft a string like `maths,physique,svt` with all the categories you want to download
4. Run `uv run main.py <link> <cookies> <categories string>` and wait for the download to finish; the files will be in the `output` folder
