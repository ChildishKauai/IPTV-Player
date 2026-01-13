# Contributing to Football Fixtures Scraper

Thank you for your interest in contributing! This project helps IPTV users find football broadcast information.

## Ways to Contribute

### 1. Report Issues
- Found a bug? Open an issue on GitHub
- Website structure changed? Let us know
- Feature request? Describe what you need

### 2. Add Features
- Support for more leagues
- Additional countries/channels
- Export formats (M3U, XMLTV)
- Web interface
- Mobile app integration

### 3. Improve Documentation
- Better installation instructions
- IPTV integration examples
- Video tutorials

### 4. Fix Bugs
- Check open issues
- Submit pull requests with fixes
- Include test cases

## Development Setup

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/football-fixtures-scraper.git
   ```
3. Create a virtual environment:
   ```bash
   python -m venv .venv
   .venv\Scripts\activate  # Windows
   ```
4. Install dependencies:
   ```bash
   pip install -r requirements.txt
   ```
5. Make your changes
6. Test thoroughly
7. Submit a pull request

## Code Style

- Follow PEP 8 Python style guide
- Use meaningful variable names
- Add comments for complex logic
- Keep functions focused and small

## Adding New Leagues

To add a new league, edit `cloudflare_bypass_scraper.py`:

```python
self.leagues = {
    'your-league-slug': 'League Name',
    # Add your league here
}
```

## Testing

Before submitting:

1. Test scraping:
   ```bash
   python fixtures.py scrape
   ```
2. Test queries:
   ```bash
   python fixtures.py today
   python fixtures.py stats
   ```
3. Check database integrity
4. Verify channel mappings in `fixtures.py`

## Pull Request Process

1. Create a feature branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```
2. Make your changes
3. Commit with clear messages:
   ```bash
   git commit -m "Add support for Serie B"
   ```
4. Push to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```
5. Open a pull request on GitHub
6. Describe your changes clearly

## Channel Country Mapping

To add new channel mappings, edit `CHANNEL_COUNTRY_MAP` in `fixtures.py`:

```python
CHANNEL_COUNTRY_MAP = {
    'Channel Name': 'Country',
    'New Channel': 'Country',  # Add here
    # ...
}
```

## Questions?

- Open an issue on GitHub
- Tag it with `question`
- We'll respond as soon as possible

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

Thank you for making this project better! 🎉
