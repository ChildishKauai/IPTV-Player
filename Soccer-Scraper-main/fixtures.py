"""
Football Fixtures Scraper - Main Interface
Query and scrape football fixtures with broadcast information
"""

import sys
from datetime import date
from database_manager import FixtureDatabase
from cloudflare_bypass_scraper import CloudflareBypassScraper
import logging

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Channel to country mapping
CHANNEL_COUNTRY_MAP = {
    # USA/America
    'NBC': 'USA', 'NBC Sports': 'USA', 'Peacock': 'USA', 'Peacock Premium': 'USA',
    'USA Network': 'USA', 'Universo': 'USA', 'ESPN': 'USA', 'ESPN+': 'USA',
    'ESPN Deportes': 'USA', 'CBS Sports': 'USA', 'CBS Sports Network': 'USA',
    'CBS Sports Golazo Network': 'USA', 'Paramount+': 'USA', 'beIN Sports': 'USA',
    'beIN Sports USA': 'USA', 'beIN Sports en Español': 'USA',
    
    # UK
    'Sky Sports': 'UK', 'Sky Sports Premier League': 'UK', 'Sky Sports Main Event': 'UK',
    'Sky Sports Ultra HDR': 'UK', 'Sky Sports 4K': 'UK', 'TNT Sports': 'UK',
    'TNT Sports 1': 'UK', 'TNT Sports 2': 'UK', 'TNT Sports 3': 'UK', 'TNT Sports 4': 'UK',
    'LaLigaTV': 'UK', 'Premier Sports': 'UK', 'Premier Sports 1': 'UK', 'Premier Sports 2': 'UK',
    
    # Spain
    'DAZN España': 'Spain', 'DAZN Spain': 'Spain', 'DAZN LaLiga': 'Spain', 'DAZN1 Spain': 'Spain',
    'Movistar': 'Spain', 'Movistar+': 'Spain', 'Movistar LaLiga': 'Spain',
    'Movistar+ Deportes': 'Spain', 'Movistar+ Deportes 2': 'Spain',
    'LaLiga TV Bar': 'Spain',
    
    # Germany
    'Sky Sport': 'Germany', 'Sky Sport Premier League': 'Germany',
    'DAZN Germany': 'Germany', 'WOW': 'Germany',
    
    # Austria
    'Sky Sport Austria': 'Austria', 'DAZN Austria': 'Austria',
    
    # Italy
    'DAZN Italia': 'Italy',
    
    # Portugal
    'DAZN Portugal': 'Portugal', 'DAZN1 Portugal': 'Portugal',
    
    # Albania
    'SuperSport 2 Digitalb': 'Albania', 'SuperSport 3 Digitalb': 'Albania',
    'Tring': 'Albania', 'Tring Sport 1': 'Albania',
    
    # France
    'Canal+ France': 'France', 'Canal+ Sport': 'France',
    
    # International
    'Bet365': 'International',
    'DAZN': 'International',
}

def get_country_for_channel(channel_name):
    """Map channel name to country"""
    if channel_name in CHANNEL_COUNTRY_MAP:
        return CHANNEL_COUNTRY_MAP[channel_name]
    
    for key, country in CHANNEL_COUNTRY_MAP.items():
        if key.lower() in channel_name.lower():
            return country
    
    return 'Various'


def print_fixtures(fixtures, db, title="FIXTURES"):
    """Print fixtures with broadcast information"""
    print("\n" + "="*80)
    print(title)
    print("="*80)
    
    if not fixtures:
        print("\nNo fixtures found")
        return
    
    for fixture in fixtures:
        print(f"\n{fixture['fixture_time']} - {fixture['home_team']} vs {fixture['away_team']}")
        print(f"  Competition: {fixture['competition']}")
        print(f"  Date: {fixture['fixture_date']}")
        if fixture.get('venue'):
            print(f"  Venue: {fixture['venue']}")
        
        # Get broadcasters
        cursor = db.conn.execute("""
            SELECT DISTINCT country, channel
            FROM broadcasters
            WHERE fixture_id = ?
            ORDER BY country, channel
        """, (fixture['id'],))
        
        broadcasters = cursor.fetchall()
        
        if broadcasters:
            print(f"  Broadcasting in:")
            
            # Group by country
            by_country = {}
            for country, channel in broadcasters:
                actual_country = get_country_for_channel(channel)
                if actual_country != 'Various':
                    country = actual_country
                
                if country not in by_country:
                    by_country[country] = []
                by_country[country].append(channel)
            
            for country in sorted(by_country.keys()):
                channels = ', '.join(sorted(set(by_country[country])))
                print(f"    {country}: {channels}")
    
    print("\n" + "="*80)


def scrape_fixtures():
    """Scrape fixtures from LiveSoccerTV"""
    db = FixtureDatabase('output/fixtures.db')
    scraper = CloudflareBypassScraper()
    
    logger.info("="*80)
    logger.info("SCRAPING FIXTURES FROM LIVESOCCERTV")
    logger.info("="*80)
    
    try:
        success = scraper.scrape_all_leagues()
        
        if success and scraper.fixtures:
            logger.info(f"\n✓ Scraped {len(scraper.fixtures)} fixtures")
            
            added = db.add_fixtures_bulk(scraper.fixtures)
            db.log_scrape(added, source='LiveSoccerTV', status='success')
            
            logger.info(f"✓ Stored {added} fixtures in database")
            
            stats = db.get_stats()
            logger.info("\n" + "="*80)
            logger.info("DATABASE STATISTICS")
            logger.info("="*80)
            logger.info(f"Total fixtures: {stats['total_fixtures']}")
            logger.info(f"Total broadcast entries: {stats['total_broadcasters']}")
            logger.info(f"Countries covered: {stats['unique_countries']}")
            
            if stats['by_competition']:
                logger.info("\nFixtures by competition:")
                for comp, count in stats['by_competition'].items():
                    logger.info(f"  {comp}: {count}")
            
            logger.info("="*80)
            
        else:
            logger.warning("No fixtures found")
            db.log_scrape(0, source='LiveSoccerTV', status='no_data')
    
    except KeyboardInterrupt:
        logger.info("\nScraping interrupted")
        db.log_scrape(0, source='LiveSoccerTV', status='interrupted')
    finally:
        scraper.close()
        db.close()


def query_today():
    """Show today's fixtures"""
    db = FixtureDatabase('output/fixtures.db')
    today_str = str(date.today())
    fixtures = db.get_fixtures_by_date(today_str)
    
    title = f"TODAY'S FIXTURES - {date.today().strftime('%A, %B %d, %Y')}"
    print_fixtures(fixtures, db, title)
    db.close()


def query_tomorrow():
    """Show tomorrow's fixtures"""
    db = FixtureDatabase('output/fixtures.db')
    from datetime import timedelta
    tomorrow = date.today() + timedelta(days=1)
    fixtures = db.get_fixtures_by_date(str(tomorrow))
    
    title = f"TOMORROW'S FIXTURES - {tomorrow.strftime('%A, %B %d, %Y')}"
    print_fixtures(fixtures, db, title)
    db.close()


def query_country(country):
    """Show fixtures for a specific country"""
    db = FixtureDatabase('output/fixtures.db')
    fixtures = db.get_fixtures_by_country(country)
    
    title = f"FIXTURES BROADCASTING IN {country.upper()}"
    print_fixtures(fixtures, db, title)
    db.close()


def query_competition(competition):
    """Show fixtures for a specific competition"""
    db = FixtureDatabase('output/fixtures.db')
    fixtures = db.get_fixtures_by_competition(competition)
    
    title = f"{competition.upper()} FIXTURES"
    print_fixtures(fixtures, db, title)
    db.close()


def show_stats():
    """Show database statistics"""
    db = FixtureDatabase('output/fixtures.db')
    stats = db.get_stats()
    
    print("\n" + "="*80)
    print("DATABASE STATISTICS")
    print("="*80)
    print(f"\nTotal fixtures: {stats['total_fixtures']}")
    print(f"Total broadcast entries: {stats['total_broadcasters']}")
    print(f"Countries covered: {stats['unique_countries']}")
    
    if stats['by_competition']:
        print("\nFixtures by competition:")
        for comp, count in stats['by_competition'].items():
            print(f"  {comp}: {count}")
    
    # Show date range
    cursor = db.conn.execute("""
        SELECT MIN(fixture_date) as min_date, MAX(fixture_date) as max_date
        FROM fixtures
    """)
    dates = cursor.fetchone()
    if dates and dates[0]:
        print(f"\nDate range: {dates[0]} to {dates[1]}")
    
    # Check for duplicates
    dupe_check = db.check_for_duplicates()
    if dupe_check['has_duplicates']:
        print(f"\n⚠️  WARNING: {dupe_check['fixture_duplicates']} duplicate fixtures, "
              f"{dupe_check['broadcaster_duplicates']} duplicate broadcasters")
        print("   Run 'python fixtures.py clean' to remove duplicates")
    
    print("="*80 + "\n")
    db.close()


def check_duplicates():
    """Check for duplicate entries in database"""
    db = FixtureDatabase('output/fixtures.db')
    
    print("\n" + "="*80)
    print("CHECKING FOR DUPLICATES")
    print("="*80 + "\n")
    
    result = db.check_for_duplicates()
    
    if result['has_duplicates']:
        print(f"⚠️  Found issues:")
        print(f"   Duplicate fixtures: {result['fixture_duplicates']}")
        print(f"   Duplicate broadcasters: {result['broadcaster_duplicates']}")
        print("\n💡 Run 'python fixtures.py clean' to remove duplicates")
    else:
        print("✓ No duplicates found - database is clean!")
    
    print("="*80 + "\n")
    db.close()


def clean_database():
    """Remove duplicate entries from database"""
    db = FixtureDatabase('output/fixtures.db')
    
    print("\n" + "="*80)
    print("CLEANING DATABASE")
    print("="*80 + "\n")
    
    # Check first
    result = db.check_for_duplicates()
    
    if not result['has_duplicates']:
        print("✓ Database is already clean - no duplicates found")
        print("="*80 + "\n")
        db.close()
        return
    
    print(f"Found {result['fixture_duplicates']} duplicate fixtures and "
          f"{result['broadcaster_duplicates']} duplicate broadcasters\n")
    
    # Clean fixtures
    if result['fixture_duplicates'] > 0:
        print("Removing duplicate fixtures...")
        removed = db.remove_duplicate_fixtures()
        print(f"✓ Removed {removed} duplicate fixtures")
    
    # Clean broadcasters
    if result['broadcaster_duplicates'] > 0:
        print("Removing duplicate broadcasters...")
        removed = db.remove_duplicate_broadcasters()
        print(f"✓ Removed {removed} duplicate broadcasters")
    
    print("\n✓ Database cleanup complete!")
    print("="*80 + "\n")
    db.close()


def print_usage():
    """Print usage information"""
    print("""
Football Fixtures Scraper

Usage:
    python fixtures.py scrape              - Scrape fixtures from LiveSoccerTV
    python fixtures.py today               - Show today's fixtures
    python fixtures.py tomorrow            - Show tomorrow's fixtures
    python fixtures.py country <NAME>      - Show fixtures for a country
    python fixtures.py competition <NAME>  - Show fixtures for a competition
    python fixtures.py stats               - Show database statistics
    python fixtures.py check               - Check for duplicate entries
    python fixtures.py clean               - Remove duplicate entries

Examples:
    python fixtures.py country USA
    python fixtures.py competition "Premier League"
    python fixtures.py check
    python fixtures.py clean
    """)


def main():
    """Main entry point"""
    if len(sys.argv) < 2:
        print_usage()
        return
    
    command = sys.argv[1].lower()
    
    if command == 'scrape':
        scrape_fixtures()
    elif command == 'today':
        query_today()
    elif command == 'tomorrow':
        query_tomorrow()
    elif command == 'country':
        if len(sys.argv) < 3:
            print("Error: Please specify a country name")
            return
        query_country(sys.argv[2])
    elif command == 'competition':
        if len(sys.argv) < 3:
            print("Error: Please specify a competition name")
            return
        query_competition(' '.join(sys.argv[2:]))
    elif command == 'stats':
        show_stats()
    elif command == 'check':
        check_duplicates()
    elif command == 'clean':
        clean_database()
    else:
        print(f"Unknown command: {command}")
        print_usage()


if __name__ == "__main__":
    main()
