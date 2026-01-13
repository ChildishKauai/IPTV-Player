# Football Fixtures Scraper

A Python web scraper that fetches football/soccer fixtures and broadcast information from LiveSoccerTV, bypassing Cloudflare protection and storing data in a local SQLite database.

**Perfect for IPTV users** - Find out which matches are being broadcast and on which channels/countries, so you can configure your IPTV player accordingly.

## Features

- **Cloudflare Bypass**: Uses undetected-chromedriver to access LiveSoccerTV
- **Multi-League Support**: Premier League, La Liga, Serie A, Ligue 1, Bundesliga, Champions League
- **Database Storage**: SQLite database to avoid continuous scraping
- **Broadcast Information**: Shows which channels/countries are broadcasting each match
- **Monthly Scraping**: Captures all fixtures for the current month
- **Fast Queries**: Instant database queries without re-scraping
- **IPTV Integration**: Export data or query database for IPTV EPG integration

## Project Structure

```
webscraper/
├── fixtures.py                      # Main interface (scrape & query)
├── cloudflare_bypass_scraper.py     # Core scraper with Cloudflare bypass
├── database_manager.py              # SQLite database operations
├── requirements.txt                 # Python dependencies
├── output/
│   ├── fixtures.db                  # SQLite database
│   └── debug_*.html                 # Debug HTML files (for troubleshooting)
└── README.md                        # This file
```

## Installation

1. **Create virtual environment**:
   ```bash
   python -m venv .venv
   ```

2. **Activate virtual environment**:
   - Windows: `.venv\Scripts\activate`
   - Linux/Mac: `source .venv/bin/activate`

3. **Install dependencies**:
   ```bash
   pip install -r requirements.txt
   ```

## Usage

### Scrape Fixtures

Scrape all fixtures for the current month from LiveSoccerTV:

```bash
python fixtures.py scrape
```

This will:
- Open Chrome browser (visible)
- Bypass Cloudflare protection automatically
- Scrape fixtures from all configured leagues
- Store results in `output/fixtures.db`
- Take ~2 minutes to complete

### Query Fixtures

Once scraped, you can query the database instantly:

**Show today's fixtures:**
```bash
python fixtures.py today
```

**Show tomorrow's fixtures:**
```bash
python fixtures.py tomorrow
```

**Show fixtures for a specific country:**
```bash
python fixtures.py country USA
python fixtures.py country UK
python fixtures.py country Spain
```

**Show fixtures for a specific competition:**
```bash
python fixtures.py competition "Premier League"
python fixtures.py competition "La Liga"
python fixtures.py competition "Serie A"
```

**Show database statistics:**
```bash
python fixtures.py stats
```

## How It Works

1. **Scraping**:
   - Uses `undetected-chromedriver` to avoid detection
   - Automatically handles Cloudflare challenges
   - Parses HTML to extract match details and broadcasters
   - Filters fixtures within current month date range
   - Maps channels to countries

2. **Storage**:
   - Stores fixtures in SQLite database
   - Tracks broadcasters per fixture
   - Avoids duplicates with UNIQUE constraints
   - Logs scraping history

3. **Querying**:
   - Fast database queries (no web requests)
   - Intelligent channel-to-country mapping
   - Formatted output with broadcast details

## Target Countries

The scraper focuses on broadcast information for:
- USA/America
- UK (United Kingdom)
- Spain
- Germany
- Austria
- Albania

But captures all available broadcast information from LiveSoccerTV.

## Supported Leagues

- **Premier League** (England)
- **La Liga** (Spain)
- **Serie A** (Italy)
- **Ligue 1** (France)
- **Bundesliga** (Germany)
- **UEFA Champions League**

## Dependencies

- `undetected-chromedriver` - Bypass Cloudflare protection
- `selenium` - Browser automation
- `beautifulsoup4` - HTML parsing
- `lxml` - Fast HTML parser
- `pandas` - Data manipulation (optional)
- `python-dotenv` - Environment variables (optional)

## Example Output

```
================================================================================
TODAY'S FIXTURES - Monday, January 12, 2026
================================================================================

18:30 - Genoa vs Cagliari
  Competition: Serie A
  Date: 2026-01-12
  Broadcasting in:
    Albania: Tring, Tring Sport 1
    Germany: DAZN Germany
    International: Bet365
    Italy: DAZN Italia
```

## Notes

- **First run**: Browser will open visibly to handle Cloudflare
- **Scraping frequency**: Run once per day or when you need updated fixtures
- **Chrome required**: Uses Chrome/Chromium browser
- **Windows cleanup warning**: Harmless OSError on exit (can be ignored)

## Troubleshooting

### No fixtures found
- Check `output/debug_*.html` files to see what was scraped
- May indicate no matches scheduled or website structure changed

### Cloudflare challenge failed
- Ensure Chrome is installed and up to date
- Check internet connection
- Try running again (sometimes takes multiple attempts)

### Database locked
- Close any other programs accessing the database
- Delete `output/fixtures.db` and re-scrape
    response = requests.get(url, headers=self.headers)
    soup = BeautifulSoup(response.content, 'lxml')
    
    # Find fixtures - customize these selectors
    fixtures = soup.find_all('div', class_='your-fixture-class')
    
    for fixture in fixtures:
        data = {
            'home_team': fixture.find('span', class_='home').text,
            'away_team': fixture.find('span', class_='away').text,
            # Add more fields...
        }
        self.fixtures.append(data)
```

## Output Format

### JSON Format

## IPTV Integration

This scraper is designed to help IPTV users find broadcast information for football matches.

### Use Cases

1. **Find which channel broadcasts a specific match**
   ```bash
   python fixtures.py today
   # Shows all today's matches with channels/countries
   ```

2. **Get matches for your IPTV region**
   ```bash
   python fixtures.py country USA
   python fixtures.py country UK
   ```

3. **Query database from your IPTV app**
   - Database location: `output/fixtures.db`
   - SQLite format (easily readable by any language)
   - Tables: `fixtures`, `broadcasters`

### Database Schema

```sql
-- Fixtures table
CREATE TABLE fixtures (
    id INTEGER PRIMARY KEY,
    home_team TEXT,
    away_team TEXT,
    competition TEXT,
    fixture_date DATE,
    fixture_time TEXT,
    venue TEXT
);

-- Broadcasters table
CREATE TABLE broadcasters (
    id INTEGER PRIMARY KEY,
    fixture_id INTEGER,
    country TEXT,
    channel TEXT,
    FOREIGN KEY (fixture_id) REFERENCES fixtures (id)
);
```

### Python Integration Example

```python
import sqlite3

# Connect to database
conn = sqlite3.connect('output/fixtures.db')
cursor = conn.cursor()

# Get today's matches
cursor.execute("""
    SELECT f.fixture_time, f.home_team, f.away_team, b.channel
    FROM fixtures f
    JOIN broadcasters b ON f.id = b.fixture_id
    WHERE f.fixture_date = date('now')
    ORDER BY f.fixture_time
""")

for match in cursor.fetchall():
    time, home, away, channel = match
    print(f"{time} - {home} vs {away} on {channel}")

conn.close()
```

## License

MIT License - See LICENSE file for details
