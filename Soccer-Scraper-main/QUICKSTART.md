# Quick Start Guide

## Installation (One-time setup)

1. **Create virtual environment:**
   ```bash
   python -m venv .venv
   ```

2. **Activate virtual environment:**
   ```bash
   .venv\Scripts\activate
   ```

3. **Install dependencies:**
   ```bash
   pip install -r requirements.txt
   ```

## Usage

### First Time - Scrape Data

```bash
python fixtures.py scrape
```

Wait ~2 minutes while it:
- Opens Chrome browser
- Bypasses Cloudflare
- Scrapes all leagues
- Saves to database

### Daily Usage - Query Data

```bash
# Today's matches
python fixtures.py today

# Tomorrow's matches  
python fixtures.py tomorrow

# Matches in USA
python fixtures.py country USA

# Premier League matches
python fixtures.py competition "Premier League"

# Statistics
python fixtures.py stats
```

## Commands Reference

| Command | Description |
|---------|-------------|
| `python fixtures.py scrape` | Scrape fixtures (do this first) |
| `python fixtures.py today` | Show today's matches |
| `python fixtures.py tomorrow` | Show tomorrow's matches |
| `python fixtures.py country <NAME>` | Show matches for a country |
| `python fixtures.py competition <NAME>` | Show matches for a league |
| `python fixtures.py stats` | Show database statistics |

## Examples

**Find where Liverpool vs Manchester United is shown:**
```bash
python fixtures.py competition "Premier League"
```

**See all matches today in USA:**
```bash
python fixtures.py today
# Look for USA channels in the output
```

**Get match schedule for the week:**
```bash
python fixtures.py stats
# Shows date range and total fixtures
```

## Tips

- ✅ Scrape once per day (or when fixtures update)
- ✅ Query as many times as you want (instant results)
- ✅ Database location: `output/fixtures.db`
- ✅ Browser opens during scraping (this is normal)
- ⚠️ First scrape takes ~2 minutes

## Troubleshooting

**"No fixtures found"**: No matches scheduled for that date/competition

**Chrome error**: Make sure Chrome browser is installed

**Database locked**: Close any programs using the database

**Need fresh data**: Run `python fixtures.py scrape` again
