"""
Database Manager for Football Fixtures
Stores scraped data in SQLite database to avoid continuous scraping
"""

import sqlite3
from datetime import datetime, date
import json
import logging

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class FixtureDatabase:
    """Manage fixture data in SQLite database"""
    
    def __init__(self, db_path='fixtures.db'):
        self.db_path = db_path
        self.conn = None
        self.create_tables()
    
    def connect(self):
        """Connect to database"""
        if not self.conn:
            self.conn = sqlite3.connect(self.db_path)
            self.conn.row_factory = sqlite3.Row
        return self.conn
    
    def close(self):
        """Close database connection"""
        if self.conn:
            self.conn.close()
            self.conn = None
    
    def create_tables(self):
        """Create database tables if they don't exist"""
        conn = self.connect()
        cursor = conn.cursor()
        
        # Fixtures table
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS fixtures (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                home_team TEXT NOT NULL,
                away_team TEXT NOT NULL,
                competition TEXT NOT NULL,
                fixture_date DATE NOT NULL,
                fixture_time TEXT,
                venue TEXT,
                scraped_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(home_team, away_team, competition, fixture_date)
            )
        ''')
        
        # Broadcasters table
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS broadcasters (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                fixture_id INTEGER NOT NULL,
                country TEXT NOT NULL,
                channel TEXT NOT NULL,
                FOREIGN KEY (fixture_id) REFERENCES fixtures (id) ON DELETE CASCADE,
                UNIQUE(fixture_id, country, channel)
            )
        ''')
        
        # Scraping history table
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS scraping_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                scrape_date DATE NOT NULL,
                scrape_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                fixtures_count INTEGER,
                source TEXT,
                status TEXT
            )
        ''')
        
        # Create indexes for faster queries
        cursor.execute('''
            CREATE INDEX IF NOT EXISTS idx_fixture_date 
            ON fixtures(fixture_date)
        ''')
        
        cursor.execute('''
            CREATE INDEX IF NOT EXISTS idx_competition 
            ON fixtures(competition)
        ''')
        
        cursor.execute('''
            CREATE INDEX IF NOT EXISTS idx_broadcaster_country 
            ON broadcasters(country)
        ''')
        
        conn.commit()
        logger.info(f"Database initialized: {self.db_path}")
    
    def add_fixture(self, fixture_data):
        """
        Add or update a fixture in the database
        
        Args:
            fixture_data: Dict with keys: home_team, away_team, competition, 
                         date, time, venue, broadcasters
        
        Returns:
            fixture_id
        """
        conn = self.connect()
        cursor = conn.cursor()
        
        try:
            # Check if fixture already exists
            cursor.execute('''
                SELECT id FROM fixtures 
                WHERE home_team = ? AND away_team = ? 
                AND competition = ? AND fixture_date = ?
            ''', (
                fixture_data.get('home_team'),
                fixture_data.get('away_team'),
                fixture_data.get('competition'),
                fixture_data.get('date')
            ))
            
            existing = cursor.fetchone()
            is_update = existing is not None
            
            # Insert or update fixture
            cursor.execute('''
                INSERT INTO fixtures (home_team, away_team, competition, 
                                    fixture_date, fixture_time, venue, last_updated)
                VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
                ON CONFLICT(home_team, away_team, competition, fixture_date)
                DO UPDATE SET
                    fixture_time = excluded.fixture_time,
                    venue = excluded.venue,
                    last_updated = CURRENT_TIMESTAMP
            ''', (
                fixture_data.get('home_team'),
                fixture_data.get('away_team'),
                fixture_data.get('competition'),
                fixture_data.get('date'),
                fixture_data.get('time'),
                fixture_data.get('venue')
            ))
            
            # Get fixture_id
            cursor.execute('''
                SELECT id FROM fixtures 
                WHERE home_team = ? AND away_team = ? 
                AND competition = ? AND fixture_date = ?
            ''', (
                fixture_data.get('home_team'),
                fixture_data.get('away_team'),
                fixture_data.get('competition'),
                fixture_data.get('date')
            ))
            
            fixture_id = cursor.fetchone()[0]
            
            # Delete existing broadcasters for this fixture to avoid duplicates
            cursor.execute('DELETE FROM broadcasters WHERE fixture_id = ?', (fixture_id,))
            
            # Add broadcasters (fresh set, no duplicates)
            broadcasters_added = 0
            for broadcaster in fixture_data.get('broadcasters', []):
                cursor.execute('''
                    INSERT OR IGNORE INTO broadcasters (fixture_id, country, channel)
                    VALUES (?, ?, ?)
                ''', (
                    fixture_id,
                    broadcaster.get('country'),
                    broadcaster.get('channel')
                ))
                if cursor.rowcount > 0:
                    broadcasters_added += 1
            
            conn.commit()
            return fixture_id
            
        except Exception as e:
            conn.rollback()
            logger.error(f"Error adding fixture: {e}")
            return None
    
    def add_fixtures_bulk(self, fixtures_list):
        """Add multiple fixtures at once, tracking new vs updated records"""
        conn = self.connect()
        cursor = conn.cursor()
        
        new_fixtures = 0
        updated_fixtures = 0
        errors = 0
        
        for fixture in fixtures_list:
            try:
                # Check if fixture already exists
                cursor.execute('''
                    SELECT id FROM fixtures 
                    WHERE home_team = ? AND away_team = ? 
                    AND competition = ? AND fixture_date = ?
                ''', (
                    fixture.get('home_team'),
                    fixture.get('away_team'),
                    fixture.get('competition'),
                    fixture.get('date')
                ))
                
                existing = cursor.fetchone()
                
                if self.add_fixture(fixture):
                    if existing:
                        updated_fixtures += 1
                    else:
                        new_fixtures += 1
                        
            except Exception as e:
                errors += 1
                logger.debug(f"Error processing fixture: {e}")
        
        total = new_fixtures + updated_fixtures
        logger.info(f"Database update: {new_fixtures} new, {updated_fixtures} updated, {errors} errors (Total: {total} fixtures)")
        return total
    
    def get_fixtures_by_date(self, target_date=None, country=None):
        """
        Get fixtures for a specific date
        
        Args:
            target_date: Date string (YYYY-MM-DD) or None for today
            country: Filter by country or None for all
        
        Returns:
            List of fixtures with broadcasters
        """
        if target_date is None:
            target_date = str(date.today())
        
        conn = self.connect()
        cursor = conn.cursor()
        
        if country:
            query = '''
                SELECT DISTINCT f.* FROM fixtures f
                JOIN broadcasters b ON f.id = b.fixture_id
                WHERE f.fixture_date = ? AND UPPER(b.country) LIKE ?
                ORDER BY f.fixture_time
            '''
            cursor.execute(query, (target_date, f'%{country.upper()}%'))
        else:
            cursor.execute('''
                SELECT * FROM fixtures
                WHERE fixture_date = ?
                ORDER BY fixture_time
            ''', (target_date,))
        
        fixtures = []
        for row in cursor.fetchall():
            fixture = dict(row)
            
            # Get broadcasters for this fixture
            cursor.execute('''
                SELECT country, channel FROM broadcasters
                WHERE fixture_id = ?
            ''', (fixture['id'],))
            
            broadcasters = [
                {'country': b['country'], 'channel': b['channel']}
                for b in cursor.fetchall()
            ]
            
            fixture['broadcasters'] = broadcasters
            fixtures.append(fixture)
        
        return fixtures
    
    def get_fixtures_by_competition(self, competition, country=None):
        """Get all fixtures for a specific competition"""
        conn = self.connect()
        cursor = conn.cursor()
        
        if country:
            query = '''
                SELECT DISTINCT f.* FROM fixtures f
                JOIN broadcasters b ON f.id = b.fixture_id
                WHERE f.competition = ? AND UPPER(b.country) LIKE ?
                ORDER BY f.fixture_date, f.fixture_time
            '''
            cursor.execute(query, (competition, f'%{country.upper()}%'))
        else:
            cursor.execute('''
                SELECT * FROM fixtures
                WHERE competition = ?
                ORDER BY fixture_date, fixture_time
            ''', (competition,))
        
        fixtures = []
        for row in cursor.fetchall():
            fixture = dict(row)
            
            # Get broadcasters
            cursor.execute('''
                SELECT country, channel FROM broadcasters
                WHERE fixture_id = ?
            ''', (fixture['id'],))
            
            broadcasters = [
                {'country': b['country'], 'channel': b['channel']}
                for b in cursor.fetchall()
            ]
            
            fixture['broadcasters'] = broadcasters
            fixtures.append(fixture)
        
        return fixtures
    
    def get_fixtures_by_country(self, country):
        """Get all fixtures available in a specific country"""
        conn = self.connect()
        cursor = conn.cursor()
        
        cursor.execute('''
            SELECT f.*, b.country, b.channel FROM fixtures f
            JOIN broadcasters b ON f.id = b.fixture_id
            WHERE UPPER(b.country) LIKE ?
            ORDER BY f.fixture_date, f.fixture_time
        ''', (f'%{country.upper()}%',))
        
        results = cursor.fetchall()
        
        # Group by fixture
        fixtures_dict = {}
        for row in results:
            fixture_id = row['id']
            
            if fixture_id not in fixtures_dict:
                fixtures_dict[fixture_id] = {
                    'id': row['id'],
                    'home_team': row['home_team'],
                    'away_team': row['away_team'],
                    'competition': row['competition'],
                    'fixture_date': row['fixture_date'],
                    'fixture_time': row['fixture_time'],
                    'venue': row['venue'],
                    'broadcasters': []
                }
            
            fixtures_dict[fixture_id]['broadcasters'].append({
                'country': row['country'],
                'channel': row['channel']
            })
        
        return list(fixtures_dict.values())
    
    def log_scrape(self, fixtures_count, source='LiveSoccerTV', status='success'):
        """Log scraping activity"""
        conn = self.connect()
        cursor = conn.cursor()
        
        cursor.execute('''
            INSERT INTO scraping_history (scrape_date, fixtures_count, source, status)
            VALUES (?, ?, ?, ?)
        ''', (str(date.today()), fixtures_count, source, status))
        
        conn.commit()
    
    def get_scraping_history(self, limit=10):
        """Get recent scraping history"""
        conn = self.connect()
        cursor = conn.cursor()
        
        cursor.execute('''
            SELECT * FROM scraping_history
            ORDER BY scrape_time DESC
            LIMIT ?
        ''', (limit,))
        
        return [dict(row) for row in cursor.fetchall()]
    
    def export_to_json(self, output_file, country=None, target_date=None):
        """Export fixtures to JSON file"""
        if target_date:
            fixtures = self.get_fixtures_by_date(target_date, country)
        else:
            fixtures = self.get_fixtures_by_country(country) if country else []
        
        with open(output_file, 'w', encoding='utf-8') as f:
            json.dump(fixtures, f, indent=2, ensure_ascii=False)
        
        logger.info(f"Exported {len(fixtures)} fixtures to {output_file}")
        return len(fixtures)
    
    def get_stats(self):
        """Get database statistics"""
        conn = self.connect()
        cursor = conn.cursor()
        
        # Total fixtures
        cursor.execute('SELECT COUNT(*) as count FROM fixtures')
        total_fixtures = cursor.fetchone()['count']
        
        # Fixtures by competition
        cursor.execute('''
            SELECT competition, COUNT(*) as count 
            FROM fixtures 
            GROUP BY competition
        ''')
        by_competition = dict(cursor.fetchall())
        
        # Total broadcasters
        cursor.execute('SELECT COUNT(*) as count FROM broadcasters')
        total_broadcasters = cursor.fetchone()['count']
        
        # Unique countries
        cursor.execute('SELECT COUNT(DISTINCT country) as count FROM broadcasters')
        unique_countries = cursor.fetchone()['count']
        
        # Last scrape
        cursor.execute('''
            SELECT scrape_time, fixtures_count 
            FROM scraping_history 
            ORDER BY scrape_time DESC 
            LIMIT 1
        ''')
        last_scrape = cursor.fetchone()
        
        return {
            'total_fixtures': total_fixtures,
            'by_competition': by_competition,
            'total_broadcasters': total_broadcasters,
            'unique_countries': unique_countries,
            'last_scrape': dict(last_scrape) if last_scrape else None
        }
    
    def clear_old_fixtures(self, days_old=30):
        """Delete fixtures older than specified days"""
        conn = self.connect()
        cursor = conn.cursor()
        
        cursor.execute('''
            DELETE FROM fixtures
            WHERE fixture_date < date('now', '-' || ? || ' days')
        ''', (days_old,))
        
        deleted = cursor.rowcount
        conn.commit()
        
        logger.info(f"Deleted {deleted} fixtures older than {days_old} days")
        return deleted
    
    def remove_duplicate_fixtures(self):
        """Remove any duplicate fixtures (shouldn't happen with UNIQUE constraint)"""
        conn = self.connect()
        cursor = conn.cursor()
        
        # Find duplicates (keep the most recent one)
        cursor.execute('''
            DELETE FROM fixtures
            WHERE id NOT IN (
                SELECT MAX(id)
                FROM fixtures
                GROUP BY home_team, away_team, competition, fixture_date
            )
        ''')
        
        deleted = cursor.rowcount
        conn.commit()
        
        if deleted > 0:
            logger.info(f"Removed {deleted} duplicate fixtures")
        else:
            logger.info("No duplicate fixtures found")
        
        return deleted
    
    def remove_duplicate_broadcasters(self):
        """Remove any duplicate broadcaster entries"""
        conn = self.connect()
        cursor = conn.cursor()
        
        # Find duplicates (keep the most recent one)
        cursor.execute('''
            DELETE FROM broadcasters
            WHERE id NOT IN (
                SELECT MAX(id)
                FROM broadcasters
                GROUP BY fixture_id, country, channel
            )
        ''')
        
        deleted = cursor.rowcount
        conn.commit()
        
        if deleted > 0:
            logger.info(f"Removed {deleted} duplicate broadcaster entries")
        else:
            logger.info("No duplicate broadcasters found")
        
        return deleted
    
    def check_for_duplicates(self):
        """Check if there are any duplicates in the database"""
        conn = self.connect()
        cursor = conn.cursor()
        
        # Check fixture duplicates
        cursor.execute('''
            SELECT home_team, away_team, competition, fixture_date, COUNT(*) as count
            FROM fixtures
            GROUP BY home_team, away_team, competition, fixture_date
            HAVING count > 1
        ''')
        
        fixture_dupes = cursor.fetchall()
        
        # Check broadcaster duplicates
        cursor.execute('''
            SELECT fixture_id, country, channel, COUNT(*) as count
            FROM broadcasters
            GROUP BY fixture_id, country, channel
            HAVING count > 1
        ''')
        
        broadcaster_dupes = cursor.fetchall()
        
        if fixture_dupes or broadcaster_dupes:
            logger.warning(f"Found {len(fixture_dupes)} duplicate fixtures and {len(broadcaster_dupes)} duplicate broadcasters")
            return {
                'fixture_duplicates': len(fixture_dupes),
                'broadcaster_duplicates': len(broadcaster_dupes),
                'has_duplicates': True
            }
        else:
            logger.info("✓ No duplicates found in database")
            return {
                'fixture_duplicates': 0,
                'broadcaster_duplicates': 0,
                'has_duplicates': False
            }


def main():
    """Example usage"""
    db = FixtureDatabase('output/fixtures.db')
    
    # Example: Add sample fixtures
    sample_fixtures = [
        {
            'home_team': 'Arsenal',
            'away_team': 'Chelsea',
            'competition': 'Premier League',
            'date': '2026-01-15',
            'time': '20:00',
            'venue': 'Emirates Stadium',
            'broadcasters': [
                {'country': 'UK', 'channel': 'Sky Sports'},
                {'country': 'USA', 'channel': 'NBC Sports'}
            ]
        }
    ]
    
    db.add_fixtures_bulk(sample_fixtures)
    
    # Get stats
    stats = db.get_stats()
    logger.info(f"\nDatabase Statistics:")
    logger.info(f"  Total fixtures: {stats['total_fixtures']}")
    logger.info(f"  Total broadcasters: {stats['total_broadcasters']}")
    logger.info(f"  Unique countries: {stats['unique_countries']}")
    
    # Close connection
    db.close()
    logger.info("\nDatabase connection closed")


if __name__ == "__main__":
    main()
