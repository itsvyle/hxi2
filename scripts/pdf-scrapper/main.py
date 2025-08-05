import os
import requests
from urllib.parse import urljoin, urlparse
from bs4 import BeautifulSoup
import argparse

# Use a common browser User-Agent string
HEADERS = {
    "User-Agent": (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
        "AppleWebKit/537.36 (KHTML, like Gecko) "
        "Chrome/115.0.0.0 Safari/537.36"
    )
}


def download_pdfs_from_url(website_url, output_dir="output"):
    # Create output directory if it doesn't exist
    os.makedirs(output_dir, exist_ok=True)

    try:
        response = requests.get(website_url, headers=HEADERS)
        response.raise_for_status()
    except requests.RequestException as e:
        print(f"Failed to fetch the website: {e}")
        return

    soup = BeautifulSoup(response.text, "html.parser")
    links = soup.find_all("a", href=True)

    pdf_links = []
    for link in links:
        href = link["href"]
        if href.lower().endswith(".pdf"):
            full_url = urljoin(website_url, href)
            pdf_links.append(full_url)

    if not pdf_links:
        print("No PDF links found.")
        return

    print(f"Found {len(pdf_links)} PDF(s). Downloading...")

    for pdf_url in pdf_links:
        try:
            pdf_response = requests.get(pdf_url)
            pdf_response.raise_for_status()

            filename = os.path.basename(urlparse(pdf_url).path)
            if not filename:
                print(f"Skipping invalid PDF URL: {pdf_url}")
                continue

            file_path = os.path.join(output_dir, filename)

            with open(file_path, "wb") as f:
                f.write(pdf_response.content)

            print(f"Saved: {file_path}")

        except requests.RequestException as e:
            print(f"Failed to download {pdf_url}: {e}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Download all PDFs from a website."
    )
    parser.add_argument(
        "--website", "-w", help="Website URL to scrape for PDFs"
    )
    parser.add_argument(
        "--output", "-o", default=".", help="Output directory (default: '.')"
    )
    args = parser.parse_args()

    if args.website:
        website = args.website
    else:
        website = input("Enter website URL: ").strip()

    if args.output:
        output_directory = os.path.join("output", args.output)
    else:
        output_directory = os.path.join(
            "output",
            input("Enter output directory (default: '.'): ").strip() or ".",
        )

    download_pdfs_from_url(website, output_directory)
