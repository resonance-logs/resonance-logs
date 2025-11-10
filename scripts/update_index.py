#!/usr/bin/env python3
"""
Update public/index.html using data from public/updater.json.
This script is intended to be run inside CI (python:3.11 image).
"""
import json
import os
import sys
import html
import re
from datetime import datetime

try:
    from bs4 import BeautifulSoup
except Exception as e:
    print("BeautifulSoup (bs4) is required. Install with: pip install beautifulsoup4", file=sys.stderr)
    raise


UPDATER = os.environ.get('UPDATER_PATH', 'public/updater.json')
INDEX = os.environ.get('INDEX_PATH', 'public/index.html')


def notes_to_html(txt: str) -> str:
    t = txt.replace('\r\n', '\n').strip()
    if not t:
        return '<em>No release notes provided.</em>'
    paras = [p.strip() for p in re.split(r'\n\s*\n', t)]
    out = []
    for p in paras:
        # convert Markdown-like links [text](url) -> <a href="url">text</a>
        p = re.sub(r'\[([^\]]+)\]\((https?://[^)]+)\)', r'<a href="\2">\1</a>', html.escape(p))
        p = p.replace('\n', '<br/>')
        out.append(f'<p>{p}</p>')
    return ''.join(out)


def format_date(iso: str) -> str:
    if not iso:
        return ''
    try:
        dt = datetime.fromisoformat(iso.replace('Z', '+00:00'))
        return dt.strftime('%Y-%m-%d %H:%M UTC')
    except Exception:
        return iso


def main():
    if not os.path.exists(UPDATER):
        print(f"Updater not found: {UPDATER}; skipping HTML update.")
        return 0

    with open(UPDATER, 'r', encoding='utf-8') as f:
        data = json.load(f)

    # Prefer linux platform key if present, else use first platform or root-level url
    platforms = data.get('platforms', {}) or {}
    platform_key = None
    for k in platforms.keys():
        if 'linux' in k.lower():
            platform_key = k
            break
    if not platform_key and platforms:
        platform_key = next(iter(platforms))
    platform_data = platforms.get(platform_key, {}) if platform_key else {}

    url = platform_data.get('url') or data.get('url') or ''
    signature = platform_data.get('signature') or ''
    version = data.get('version') or ''
    notes = data.get('notes') or ''
    pub_date = data.get('pub_date') or data.get('date') or ''
    pretty_date = format_date(pub_date)

    notes_html = notes_to_html(notes)

    # read and modify index.html
    if not os.path.exists(INDEX):
        print(f"Index file not found: {INDEX}")
        return 1

    with open(INDEX, 'r', encoding='utf-8') as f:
        original = f.read()

    # preserve doctype if present at top
    doctype_match = re.match(r'(?is)(<!doctype[^>]*>\s*)', original)
    doctype = doctype_match.group(1) if doctype_match else ''

    soup = BeautifulSoup(original, 'html.parser')

    # Update download link/button
    dl = soup.find(id='download-latest')
    if dl and url:
        dl['href'] = url
        if version:
            dl.string = f'Download Latest ({version})'
        else:
            dl.string = 'Download Latest'
        if 'download' in dl.attrs and url.startswith('http'):
            del dl.attrs['download']

    # Update latest info
    info = soup.find(id='latest-info')
    if info is not None:
        info.clear()
        header_html = f"<strong>Latest: {html.escape(version)}</strong> — released {html.escape(pretty_date)}"
        content_html = header_html + '<br/><br/>' + '<strong>Release notes:</strong>' + notes_html + '<br/>' + '<strong>Signature:</strong>' + f"<pre class='signature' style='white-space:pre-wrap;background:rgba(255,255,255,0.02);padding:8px;border-radius:6px'>{html.escape(signature)}</pre>"
        info.append(BeautifulSoup(content_html, 'html.parser'))

    # Populate changelog-list with single entry
    clist = soup.find(id='changelog-list')
    if clist is not None:
        clist.clear()
        li = soup.new_tag('li', **{'class': 'release'})
        left = soup.new_tag('div')
        title = soup.new_tag('div')
        title.append(BeautifulSoup(f"<strong>{html.escape(version)}</strong> <span class='meta'>{html.escape(pretty_date)}</span>", 'html.parser'))
        notes_div = soup.new_tag('div', **{'class': 'meta'})
        notes_div.append(BeautifulSoup(notes_html, 'html.parser'))
        left.append(title)
        left.append(notes_div)

        right = soup.new_tag('div')
        dlk = soup.new_tag('a', **{'class': 'button', 'href': url or '#'})
        dlk.string = 'Download'
        if url and url.startswith('/'):
            dlk['download'] = ''
        right.append(dlk)

        li.append(left)
        li.append(right)
        clist.append(li)

        loading = soup.find(id='changelog-loading')
        if loading is not None:
            loading['hidden'] = 'true'
        if 'hidden' in clist.attrs:
            del clist.attrs['hidden']

    out = str(soup)
    if doctype and not out.lower().lstrip().startswith('<!doctype'):
        out = doctype + out

    with open(INDEX, 'w', encoding='utf-8') as f:
        f.write(out)

    print(f"Updated '{INDEX}' with data from '{UPDATER}' (platform={platform_key})")
    return 0


if __name__ == '__main__':
    sys.exit(main())
