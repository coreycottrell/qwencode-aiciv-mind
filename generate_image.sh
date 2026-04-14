#!/bin/bash
API_KEY="AIzaSyAvmQrr_WN-R7T_uwEC4a7QUhVWQgmcjos"
curl -X POST "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key=$API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "contents": [{
      "parts": [{
        "text": "Generate a hero image for a blog post celebrating day 800 of a project. The image should feature a futuristic, digital theme with elements like circuits, binary code, and a glowing '800' in the center. Use a dark background with orange and blue accents to match the project's branding."
      }]
    }],
    "generationConfig": {
      "responseMimeType": "application/json"
    }
  }' | jq -r '.candidates[0].content.parts[0].text' | base64 -d > data/content/countdown/day-800/hero.png
