"""
Web Search Bridge for Luna (Windows Virtual Secretary)
Uses duckduckgo-search (ddgs) to fetch web search results.
"""

import sys
import json


def perform_search(query: str, max_results: int = 5):
    """
    Perform web search using duckduckgo_search DDGS.
    """
    if not query.strip():
        return []
    try:
        from duckduckgo_search import DDGS
        with DDGS() as ddgs:
            raw_results = list(ddgs.text(query, max_results=max_results))
            items = []
            for r in raw_results:
                items.append({
                    "title": r.get("title", ""),
                    "href": r.get("href", r.get("link", "")),
                    "body": r.get("body", r.get("snippet", ""))
                })
            return items
    except Exception as e:
        sys.stderr.write(f"DDGS search error: {e}\n")
        return []


def main():
    """
    Main entrypoint for Web Search subprocess call.
    """
    query = sys.argv[1] if len(sys.argv) > 1 else ""
    results = perform_search(query)
    status = "ok" if results else "no_results"
    print(json.dumps({"query": query, "results": results, "status": status}))


if __name__ == "__main__":
    main()
