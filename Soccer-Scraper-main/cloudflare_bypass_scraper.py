"""
LiveSoccerTV Scraper with Cloudflare Bypass
Uses undetected-chromedriver to bypass Cloudflare's "Verifying you are human" protection
"""

import undetected_chromedriver as uc
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from bs4 import BeautifulSoup
import json
import time
from datetime import datetime, date, timedelta
import logging
import warnings
import re

# Suppress the harmless undetected_chromedriver cleanup warning on Windows
warnings.filterwarnings("ignore", category=ResourceWarning)

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class CloudflareBypassScraper:
    """Scrape LiveSoccerTV with Cloudflare bypass"""
    
    def __init__(self):
        self.driver = None
        self.fixtures = []
        self.today = date.today()
        # Calculate month range
        self.month_start = self.today.replace(day=1)
        last_day = 31
        while True:
            try:
                self.month_end = self.today.replace(day=last_day)
                break
            except ValueError:
                last_day -= 1
        
        logger.info(f"Scraping fixtures from {self.month_start} to {self.month_end}")
        
        self.target_countries = ['USA', 'AMERICA', 'SPAIN', 'GERMANY', 'AUSTRIA', 'ALBANIA', 'UK']
        self.leagues = {
            'england/premier-league': 'Premier League',
            'italy/serie-a': 'Serie A',
            'france/ligue-1': 'Ligue 1',
            'spain/primera-division': 'La Liga',
            'germany/bundesliga/': 'Bundesliga',
            'international/uefa-champions-league/': 'UEFA Champions League',
        }
    
    def setup_driver(self):
        """Set up undetected Chrome driver to bypass Cloudflare"""
        try:
            options = uc.ChromeOptions()
            
            # Basic options
            options.add_argument('--disable-blink-features=AutomationControlled')
            options.add_argument('--disable-dev-shm-usage')
            options.add_argument('--no-sandbox')
            options.add_argument('--window-size=1920,1080')
            
            # Optional: Run headless (may not work with all Cloudflare challenges)
            # options.add_argument('--headless=new')
            
            # Initialize undetected chromedriver
            self.driver = uc.Chrome(options=options, version_main=143)
            
            logger.info("Undetected Chrome driver initialized successfully")
            logger.info("This driver can bypass Cloudflare protection")
            return True
            
        except Exception as e:
            logger.error(f"Error setting up Chrome driver: {e}")
            return False
    
    def wait_for_cloudflare(self, timeout=30):
        """Wait for Cloudflare challenge to complete"""
        logger.info("Waiting for Cloudflare challenge to complete...")
        time.sleep(5)  # Initial wait
        
        start_time = time.time()
        while time.time() - start_time < timeout:
            page_source = self.driver.page_source.lower()
            
            # Check if Cloudflare challenge is present
            if 'cloudflare' in page_source and ('checking your browser' in page_source or 'just a moment' in page_source):
                logger.info("Cloudflare challenge detected, waiting...")
                time.sleep(2)
            else:
                logger.info("✓ Cloudflare challenge passed!")
                return True
        
        logger.warning("Cloudflare challenge timeout")
        return False
    
    def scrape_league(self, league_slug: str, league_name: str):
        """Scrape fixtures from a specific league"""
        url = f"https://www.livesoccertv.com/competitions/{league_slug}/"
        logger.info(f"\nScraping {league_name} from: {url}")
        
        try:
            self.driver.get(url)
            
            # Wait for Cloudflare
            self.wait_for_cloudflare()
            
            # Reduced wait for page load (optimized)
            time.sleep(1.5)
            
            # Get page source and parse with BeautifulSoup
            soup = BeautifulSoup(self.driver.page_source, 'lxml')
            
            # Debug: Save page source to check structure (disabled for performance)
            # with open(f'output/debug_{league_name.replace(" ", "_")}.html', 'w', encoding='utf-8') as f:
            #     f.write(self.driver.page_source)
            # logger.info(f"Saved debug HTML to output/debug_{league_name.replace(' ', '_')}.html")
            
            # Look for different possible fixture containers
            fixtures_found = 0
            
            # Method 1: Look for match rows (tr with class matchrow)
            match_rows = soup.find_all('tr', class_='matchrow')
            logger.info(f"Found {len(match_rows)} match rows")
            
            for row in match_rows:
                fixture_data = self._parse_match_row(row, league_name)
                if fixture_data:
                    self.fixtures.append(fixture_data)
                    fixtures_found += 1
            
            # Method 2: Look for schedule tables (fallback)
            if fixtures_found == 0:
                tables = soup.find_all('table', class_='schedules')
                logger.info(f"Found {len(tables)} schedule tables")
                
                for table in tables:
                    rows = table.find_all('tr')
                    for row in rows:
                        fixture_data = self._parse_fixture_row(row, league_name)
                        if fixture_data:
                            self.fixtures.append(fixture_data)
                            fixtures_found += 1
            
            # Method 3: Look for match items in divs
            if fixtures_found == 0:
                match_divs = soup.find_all('div', class_=['match', 'fixture', 'game'])
                logger.info(f"Found {len(match_divs)} match divs")
                
                for div in match_divs:
                    fixture_data = self._parse_fixture_div(div, league_name)
                    if fixture_data:
                        self.fixtures.append(fixture_data)
                        fixtures_found += 1
            
            logger.info(f"✓ Found {fixtures_found} fixtures for {league_name}")
            
        except Exception as e:
            logger.error(f"Error scraping {league_name}: {e}")
    
    def _parse_match_row(self, row, league_name):
        """Parse a match row with class='matchrow' from LiveSoccerTV"""
        try:
            # Extract match link
            match_link = row.find('a', href=lambda x: x and '/match/' in str(x))
            if not match_link:
                return None
            
            match_text = match_link.get_text(strip=True)
            
            # Parse teams from match text (format: "Team1 vs Team2" or "Team1 0-1 Team2")
            # Remove score if present
            score_pattern = r'\s+\d+\s*-\s*\d+\s+'
            match_text_clean = re.sub(score_pattern, ' vs ', match_text)
            
            if ' vs ' not in match_text_clean:
                return None
            
            teams = match_text_clean.split(' vs ')
            if len(teams) != 2:
                return None
            
            home_team = teams[0].strip()
            away_team = teams[1].strip()
            
            # Extract date from date row (previous sibling tr with class drow)
            fixture_date = str(self.today)
            date_row = row.find_previous_sibling('tr', class_='drow')
            if date_row:
                date_link = date_row.find('a', href=lambda x: x and '/schedules/' in str(x))
                if date_link:
                    href = date_link.get('href')
                    # Extract date from URL like /schedules/2026-01-17/
                    date_match = re.search(r'/schedules/(\d{4}-\d{2}-\d{2})/', href)
                    if date_match:
                        fixture_date = date_match.group(1)
            
            # Extract time
            time_span = row.find('span', class_='ts')
            fixture_time = 'TBD'
            if time_span:
                fixture_time = time_span.get_text(strip=True)
            
            # Extract broadcasters
            broadcasters = []
            channels_td = row.find('td', id='channels')
            if channels_td:
                channel_links = channels_td.find_all('a', href=lambda x: x and '/channels/' in str(x))
                for link in channel_links:
                    channel_name = link.get_text(strip=True)
                    # Try to find country from channel name or class
                    # For now, we'll mark as 'Various' and filter later based on known channels
                    if channel_name and len(channel_name) > 1:
                        broadcasters.append({
                            'country': 'Various',  # Will be enhanced with country mapping
                            'channel': channel_name
                        })
            
            fixture = {
                'home_team': home_team,
                'away_team': away_team,
                'competition': league_name,
                'date': fixture_date,
                'time': fixture_time,
                'broadcasters': broadcasters,
                'scraped_at': datetime.now().isoformat()
            }
            
            # Check if within month range
            fixture_date_obj = datetime.strptime(fixture_date, '%Y-%m-%d').date()
            if self.month_start <= fixture_date_obj <= self.month_end:
                return fixture
            
            return None
            
        except Exception as e:
            logger.debug(f"Error parsing match row: {e}")
            return None
    
    def _parse_fixture_row(self, row, league_name):
        """Parse a single fixture row from table"""
        try:
            # Look for team links
            team_links = row.find_all('a', href=lambda x: x and '/teams/' in str(x))
            
            if len(team_links) < 2:
                return None
            
            # Extract date - look for date cell
            date_cell = row.find('td', class_='date') or row.find('span', class_='date')
            fixture_date = str(self.today)  # default
            
            if date_cell:
                date_text = date_cell.get_text(strip=True)
                parsed_date = self._parse_date_text(date_text)
                if parsed_date:
                    fixture_date = str(parsed_date)
            
            fixture = {
                'home_team': team_links[0].get_text(strip=True),
                'away_team': team_links[1].get_text(strip=True),
                'competition': league_name,
                'date': fixture_date,
                'broadcasters': []
            }
            
            # Extract time
            time_elem = row.find('span', class_='time') or row.find('td', class_='time')
            if time_elem:
                fixture['time'] = time_elem.get_text(strip=True)
            else:
                fixture['time'] = 'TBD'
            
            # Extract venue if available
            venue_elem = row.find('span', class_='venue') or row.find('td', class_='venue')
            if venue_elem:
                fixture['venue'] = venue_elem.get_text(strip=True)
            
            # Extract broadcasters
            broadcaster_cells = row.find_all('td', class_='broadcaster')
            
            for cell in broadcaster_cells:
                # Get country from flag image
                country_img = cell.find('img', alt=True)
                country = country_img.get('alt', 'Unknown') if country_img else 'Various'
                
                # Get channel names
                channel_links = cell.find_all('a')
                for link in channel_links:
                    channel_name = link.get_text(strip=True)
                    if channel_name and len(channel_name) > 1:
                        fixture['broadcasters'].append({
                            'country': country,
                            'channel': channel_name
                        })
            
            fixture['scraped_at'] = datetime.now().isoformat()
            
            if fixture['home_team'] and fixture['away_team']:
                # Only include fixtures within the current month
                fixture_date_obj = datetime.strptime(fixture_date, '%Y-%m-%d').date()
                if self.month_start <= fixture_date_obj <= self.month_end:
                    return fixture
            
            return None
            
        except Exception as e:
            logger.debug(f"Error parsing row: {e}")
            return None
    
    def _parse_date_text(self, date_text):
        """Parse various date formats from the website"""
        try:
            # Clean the text
            date_text = date_text.strip()
            
            # Handle "Today"
            if 'today' in date_text.lower():
                return self.today
            
            # Handle "Tomorrow"
            if 'tomorrow' in date_text.lower():
                return self.today + timedelta(days=1)
            
            # Handle relative dates like "Mon 13 Jan"
            current_year = self.today.year
            
            # Try parsing formats like "Mon 13 Jan" or "13 Jan"
            patterns = [
                r'(\d{1,2})\s+([A-Za-z]{3})',  # "13 Jan"
                r'[A-Za-z]{3}\s+(\d{1,2})\s+([A-Za-z]{3})',  # "Mon 13 Jan"
                r'(\d{1,2})/(\d{1,2})/(\d{4})',  # "13/01/2026"
                r'(\d{4})-(\d{1,2})-(\d{1,2})',  # "2026-01-13"
            ]
            
            for pattern in patterns:
                match = re.search(pattern, date_text)
                if match:
                    groups = match.groups()
                    if len(groups) == 2:
                        # Day and month abbreviation
                        day = int(groups[0])
                        month_abbr = groups[1]
                        month_map = {
                            'jan': 1, 'feb': 2, 'mar': 3, 'apr': 4,
                            'may': 5, 'jun': 6, 'jul': 7, 'aug': 8,
                            'sep': 9, 'oct': 10, 'nov': 11, 'dec': 12
                        }
                        month = month_map.get(month_abbr.lower()[:3])
                        if month:
                            return date(current_year, month, day)
                    elif len(groups) == 3:
                        # Full date
                        if '/' in date_text:
                            day, month, year = int(groups[0]), int(groups[1]), int(groups[2])
                        else:
                            year, month, day = int(groups[0]), int(groups[1]), int(groups[2])
                        return date(year, month, day)
            
            return None
            
        except Exception as e:
            logger.debug(f"Error parsing date '{date_text}': {e}")
            return None
    
    def _parse_fixture_div(self, div, league_name):
        """Parse fixture from div element"""
        try:
            teams = div.find_all(['span', 'div'], class_=['team', 'team-name'])
            
            if len(teams) < 2:
                return None
            
            # Try to extract date
            date_elem = div.find(['span', 'div'], class_=['date', 'match-date'])
            fixture_date = str(self.today)
            if date_elem:
                parsed_date = self._parse_date_text(date_elem.get_text(strip=True))
                if parsed_date:
                    fixture_date = str(parsed_date)
            
            fixture = {
                'home_team': teams[0].get_text(strip=True),
                'away_team': teams[1].get_text(strip=True),
                'competition': league_name,
                'date': fixture_date,
                'time': 'TBD',
                'broadcasters': [],
                'scraped_at': datetime.now().isoformat()
            }
            
            # Check if within month range
            fixture_date_obj = datetime.strptime(fixture_date, '%Y-%m-%d').date()
            if self.month_start <= fixture_date_obj <= self.month_end:
                return fixture if fixture['home_team'] and fixture['away_team'] else None
            
            return None
            
        except Exception as e:
            logger.debug(f"Error parsing div: {e}")
            return None
    
    def _parse_from_link(self, link, league_name):
        """Parse basic fixture info from match link"""
        try:
            text = link.get_text(strip=True)
            if ' vs ' in text or ' v ' in text:
                parts = text.split(' vs ' if ' vs ' in text else ' v ')
                if len(parts) == 2:
                    return {
                        'home_team': parts[0].strip(),
                        'away_team': parts[1].strip(),
                        'competition': league_name,
                        'date': str(self.today),
                        'time': 'TBD',
                        'broadcasters': [],
                        'scraped_at': datetime.now().isoformat()
                    }
            return None
        except Exception as e:
            logger.debug(f"Error parsing link: {e}")
            return None
    
    def scrape_all_leagues(self):
        """Scrape all configured leagues"""
        if not self.driver:
            if not self.setup_driver():
                logger.error("Failed to setup driver, cannot continue")
                return False
        
        for league_slug, league_name in self.leagues.items():
            self.scrape_league(league_slug, league_name)
            time.sleep(1)  # Wait between requests (optimized)
        
        return len(self.fixtures) > 0
    
    def filter_by_target_countries(self):
        """Filter fixtures to only include target countries"""
        filtered_fixtures = []
        
        for fixture in self.fixtures:
            filtered_broadcasters = []
            for bc in fixture.get('broadcasters', []):
                country = bc.get('country', '').upper()
                if any(target.upper() in country or country in target.upper() for target in self.target_countries):
                    filtered_broadcasters.append(bc)
            
            if filtered_broadcasters:
                fixture_copy = fixture.copy()
                fixture_copy['broadcasters'] = filtered_broadcasters
                filtered_fixtures.append(fixture_copy)
            elif not fixture.get('broadcasters'):
                # Include fixtures without broadcaster info
                filtered_fixtures.append(fixture.copy())
        
        return filtered_fixtures
    
    def save_to_json(self, filename='cloudflare_bypass_fixtures.json'):
        """Save filtered fixtures to JSON"""
        import os
        os.makedirs('output', exist_ok=True)
        filepath = f'output/{filename}'
        
        filtered_data = self.filter_by_target_countries()
        
        with open(filepath, 'w', encoding='utf-8') as f:
            json.dump(filtered_data, f, indent=2, ensure_ascii=False)
        
        logger.info(f"\n✓ Saved {len(filtered_data)} fixtures to {filepath}")
        
        return filtered_data
    
    def print_summary(self):
        """Print summary of scraped data"""
        filtered = self.filter_by_target_countries()
        
        logger.info("\n" + "="*80)
        logger.info(f"SCRAPING RESULTS - {self.month_start.strftime('%B %Y')}")
        logger.info(f"Date Range: {self.month_start} to {self.month_end}")
        logger.info("="*80)
        logger.info(f"Total fixtures scraped: {len(self.fixtures)}")
        logger.info(f"Fixtures with target countries: {len(filtered)}")
        
        if filtered:
            # Count by league
            for league_name in set([f['competition'] for f in filtered]):
                count = len([f for f in filtered if f['competition'] == league_name])
                logger.info(f"  {league_name}: {count} fixtures")
            
            # Count by country
            country_counts = {}
            for fixture in filtered:
                for bc in fixture.get('broadcasters', []):
                    country = bc['country']
                    country_counts[country] = country_counts.get(country, 0) + 1
            
            if country_counts:
                logger.info(f"\nBroadcasts by target country:")
                for country in sorted(country_counts.keys()):
                    logger.info(f"  {country}: {country_counts[country]} broadcasts")
        
        logger.info("="*80 + "\n")
    
    def close(self):
        """Close the browser"""
        if self.driver:
            try:
                self.driver.quit()
                logger.info("Browser closed")
            except Exception as e:
                # Suppress harmless cleanup errors
                logger.debug(f"Browser cleanup error (can be ignored): {e}")
            finally:
                self.driver = None


def main():
    """Main execution"""
    import os
    os.makedirs('output', exist_ok=True)
    
    scraper = CloudflareBypassScraper()
    
    logger.info("="*80)
    logger.info("LIVESOCCERTV SCRAPER WITH CLOUDFLARE BYPASS")
    logger.info(f"Target Countries: {', '.join(scraper.target_countries)}")
    logger.info("="*80)
    logger.info("\nUsing undetected-chromedriver to bypass Cloudflare protection")
    logger.info("The browser will open and handle Cloudflare challenges automatically\n")
    
    try:
        # Scrape all leagues
        success = scraper.scrape_all_leagues()
        
        if success and scraper.fixtures:
            # Print summary
            scraper.print_summary()
            
            # Save to JSON
            scraper.save_to_json()
            
            logger.info("✓ Scraping completed successfully!")
            logger.info("  Check the output folder for results and debug HTML files")
        else:
            logger.warning("\nNo fixtures found from scraping.")
            logger.info("This could mean:")
            logger.info("  1. There are no matches scheduled for today")
            logger.info("  2. The website structure has changed")
            logger.info("  3. Additional Cloudflare protection is in place")
            logger.info("\nCheck the debug HTML files in the output folder for more info.")
            
            # Fall back to sample data
            logger.info("\nGenerating comprehensive sample data...")
            from scraper_alternative import AlternativeFixtureScraper
            alt_scraper = AlternativeFixtureScraper()
            alt_scraper.generate_comprehensive_fixtures()
            alt_scraper.save_filtered_json('cloudflare_bypass_fixtures.json')
            alt_scraper.print_summary_by_country()
    
    except KeyboardInterrupt:
        logger.info("\n\nScraping interrupted by user")
    except Exception as e:
        logger.error(f"\nUnexpected error: {e}")
        import traceback
        traceback.print_exc()
    finally:
        # Always close the browser
        logger.info("\nClosing browser...")
        scraper.close()


if __name__ == "__main__":
    main()
