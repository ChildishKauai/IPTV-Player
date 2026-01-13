# How to Upload to GitHub

## Prerequisites

1. **Install Git** (if not already installed):
   - Download from: https://git-scm.com/download/win
   - Install with default settings
   - Restart your terminal after installation

2. **Create GitHub account** (if you don't have one):
   - Go to: https://github.com/signup
   - Create a free account

## Step 1: Initialize Git Repository

Open PowerShell in the project directory and run:

```bash
git init
git add .
git commit -m "Initial commit: Football Fixtures Scraper for IPTV"
```

## Step 2: Create GitHub Repository

1. Go to https://github.com/new
2. Fill in the details:
   - **Repository name**: `football-fixtures-scraper` (or your preferred name)
   - **Description**: `Web scraper for football fixtures and broadcast info - Perfect for IPTV users`
   - **Visibility**: Choose Public or Private
   - **DON'T** initialize with README (we already have one)
3. Click "Create repository"

## Step 3: Push to GitHub

After creating the repository, GitHub will show you commands. Run these in PowerShell:

```bash
git remote add origin https://github.com/YOUR_USERNAME/YOUR_REPO_NAME.git
git branch -M main
git push -u origin main
```

Replace `YOUR_USERNAME` and `YOUR_REPO_NAME` with your actual values.

### Example:
```bash
git remote add origin https://github.com/john/football-fixtures-scraper.git
git branch -M main
git push -u origin main
```

## Step 4: Verify Upload

Visit your repository URL:
```
https://github.com/YOUR_USERNAME/YOUR_REPO_NAME
```

You should see all your files!

## Quick Upload (Alternative Method)

If you prefer using GitHub Desktop (easier for beginners):

1. **Download GitHub Desktop**: https://desktop.github.com/
2. **Install and sign in** to your GitHub account
3. Click **File** → **Add Local Repository**
4. Select your project folder: `C:\Users\User\OneDrive\Desktop\webscraper`
5. Click **Publish repository**
6. Choose visibility (Public/Private) and click **Publish**

Done! Your project is now on GitHub.

## Using the Project for IPTV

Once uploaded, you can:

1. **Clone on any device**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/YOUR_REPO_NAME.git
   ```

2. **Share with others**: Give them your repository URL

3. **Access from IPTV devices**: 
   - If your IPTV device supports Python, clone the repo
   - Run `python fixtures.py scrape` to get data
   - Query the SQLite database from your IPTV app

4. **Keep it updated**:
   ```bash
   git add .
   git commit -m "Update fixtures"
   git push
   ```

## Useful Git Commands

```bash
# Check status
git status

# Add all files
git add .

# Commit changes
git commit -m "Your message"

# Push to GitHub
git push

# Pull latest changes
git pull
```

## Troubleshooting

### "git: command not found"
- Install Git from https://git-scm.com/download/win
- Restart your terminal

### "Permission denied"
- Use HTTPS URL (not SSH): `https://github.com/username/repo.git`
- Or set up SSH keys: https://docs.github.com/en/authentication

### "Repository not found"
- Check the repository URL is correct
- Make sure you have access (if it's private)

## Next Steps

After uploading:

1. Add a repository description on GitHub
2. Add topics/tags: `iptv`, `football`, `scraper`, `livesoccertv`, `python`
3. Share the link with IPTV communities
4. Consider adding GitHub Actions for automated scraping
