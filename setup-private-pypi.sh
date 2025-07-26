#!/bin/bash
# Quick setup for private PyPI server
set -e

echo "🔒 Setting up Private PyPI Server"
echo "================================"
echo ""

# Menu
echo "Choose private hosting method:"
echo "1) devpi (Full featured)"
echo "2) pypiserver (Simple)"
echo "3) S3 bucket setup"
echo "4) Simple HTTP server"
echo ""
read -p "Enter choice [1-4]: " choice

case $choice in
    1)
        echo "📦 Setting up devpi..."
        
        # Install devpi
        pip install devpi-server devpi-client devpi-web
        
        # Initialize
        devpi-init
        
        # Start server
        devpi-server --start --host 0.0.0.0 --port 3141
        
        # Configure
        devpi use http://localhost:3141
        devpi login root --password=""
        devpi index -c company/stable bases=root/pypi
        devpi index company/stable bases=root/pypi
        
        echo ""
        echo "✅ devpi running at http://localhost:3141"
        echo ""
        echo "Upload wheels:"
        echo "  devpi upload python/dist/*.whl"
        echo ""
        echo "Install packages:"
        echo "  pip install -i http://localhost:3141/company/stable tmux-agent"
        ;;
        
    2)
        echo "📦 Setting up pypiserver..."
        
        # Install
        pip install pypiserver passlib
        
        # Create directories
        mkdir -p ~/private-pypi/packages
        
        # Create auth file (optional)
        echo "Creating authentication..."
        htpasswd -sc ~/private-pypi/.htaccess pypi
        
        # Create start script
        cat > ~/private-pypi/start-server.sh << 'EOF'
#!/bin/bash
pypi-server -p 8080 \
    --authenticate upload \
    --passwords ~/private-pypi/.htaccess \
    ~/private-pypi/packages
EOF
        chmod +x ~/private-pypi/start-server.sh
        
        echo ""
        echo "✅ pypiserver configured"
        echo ""
        echo "Start server:"
        echo "  ~/private-pypi/start-server.sh"
        echo ""
        echo "Upload wheels:"
        echo "  cp python/dist/*.whl ~/private-pypi/packages/"
        echo ""
        echo "Install packages:"
        echo "  pip install --index-url http://localhost:8080 tmux-agent"
        ;;
        
    3)
        echo "📦 S3 Bucket Setup Helper..."
        
        # Create index generator
        cat > generate-s3-index.py << 'EOF'
#!/usr/bin/env python3
import os
import glob

print("<html><body>")
print("<h1>Python Package Index</h1>")

for wheel in glob.glob("*.whl"):
    print(f'<a href="{wheel}">{wheel}</a><br>')

print("</body></html>")
EOF
        chmod +x generate-s3-index.py
        
        echo ""
        echo "📝 S3 Setup Instructions:"
        echo ""
        echo "1. Create S3 bucket:"
        echo "   aws s3 mb s3://your-company-pypi"
        echo ""
        echo "2. Upload wheels:"
        echo "   aws s3 cp python/dist/ s3://your-company-pypi/tmux-agent/ --recursive"
        echo ""
        echo "3. Create index:"
        echo "   cd python/dist/"
        echo "   python ../../generate-s3-index.py > index.html"
        echo "   aws s3 cp index.html s3://your-company-pypi/tmux-agent/"
        echo ""
        echo "4. Set bucket policy for private access"
        echo ""
        echo "5. Install:"
        echo "   pip install --index-url https://your-company-pypi.s3.amazonaws.com/ tmux-agent"
        ;;
        
    4)
        echo "📦 Setting up simple HTTP server..."
        
        # Create directory structure
        mkdir -p ~/simple-pypi/tmux-agent
        
        # Copy wheels
        cp python/dist/*.whl ~/simple-pypi/tmux-agent/ 2>/dev/null || echo "No wheels found yet"
        
        # Create simple index
        cd ~/simple-pypi/tmux-agent
        cat > index.html << 'EOF'
<html>
<body>
<h1>tmux-agent</h1>
EOF
        
        for wheel in *.whl; do
            [ -f "$wheel" ] && echo "<a href=\"$wheel\">$wheel</a><br>" >> index.html
        done
        
        echo "</body></html>" >> index.html
        
        # Create start script
        cat > ~/simple-pypi/serve.sh << 'EOF'
#!/bin/bash
cd ~/simple-pypi
python3 -m http.server 8000
EOF
        chmod +x ~/simple-pypi/serve.sh
        
        echo ""
        echo "✅ Simple server configured"
        echo ""
        echo "Start server:"
        echo "  ~/simple-pypi/serve.sh"
        echo ""
        echo "Install packages:"
        echo "  pip install --trusted-host localhost --index-url http://localhost:8000 tmux-agent"
        ;;
esac

echo ""
echo "🔐 Security Tips:"
echo "  - Use HTTPS in production"
echo "  - Add authentication for uploads"
echo "  - Restrict network access"
echo "  - Regular security updates"