#!/usr/bin/env python3
"""
chat_with_cortex.py — Interactive chat interface for Cortex

A simple, smart way to chat with Cortex without needing bash/terminal commands.
Just run this script and type your messages - Cortex responds directly.

Features:
- Interactive chat session
- Clean formatting
- Uses the existing cortex_chat infrastructure
- No Telegram required
- Works from any directory

Usage:
  python3 chat_with_cortex.py
  (or just: chat_with_cortex)
"""

import subprocess
import sys
import os
import shlex
from pathlib import Path
from dotenv import load_dotenv, find_dotenv

# Add the tools directory to PATH if not already there

load_dotenv(find_dotenv())
def ensure_path():
    """Ensure the tools directory is in PATH."""
    tools_dir = Path("/home/corey/projects/AI-CIV/aiciv-mind-cubed/tools")
    if str(tools_dir) not in os.environ.get('PATH', ''):
        os.environ['PATH'] = f"{tools_dir}:{os.environ.get('PATH', '')}"

def run_cortex_command(message: str) -> str:
    """Run the cortex command and capture output."""
    try:
        # Use the cortex command directly
        result = subprocess.run(
            ["cortex", message],
            capture_output=True,
            text=True,
            timeout=120
        )
        
        if result.returncode != 0:
            return f"Error: {result.stderr}"
        
        return result.stdout
    except subprocess.TimeoutExpired:
        return "Error: Cortex response timed out"
    except Exception as e:
        return f"Error: {str(e)}"

def main():
    """Main chat loop."""
    ensure_path()
    
    print("🤖 Cortex Chat Interface")
    print("Type your messages below. Type 'quit', 'exit', or 'bye' to end the chat.")
    print("=" * 60)
    
    while True:
        try:
            # Get user input
            user_input = input("\n💬 You: ").strip()
            
            # Check for exit commands
            if user_input.lower() in ['quit', 'exit', 'bye']:
                print("\n👋 Goodbye!")
                break
            
            if not user_input:
                continue
            
            # Send to Cortex
            print("\n🤖 Cortex: ", end="", flush=True)
            response = run_cortex_command(user_input)
            
            # Print response with nice formatting
            lines = response.split('\n')
            for i, line in enumerate(lines):
                if i == 0:
                    print(line)
                else:
                    print(f"      {line}")
            
            print("\n" + "=" * 60)  # Separator between exchanges
            
        except KeyboardInterrupt:
            print("\n\n👋 Goodbye!")
            break
        except EOFError:
            print("\n\n👋 Goodbye!")
            break
        except Exception as e:
            print(f"\nError: {e}")
            break

if __name__ == "__main__":
    main()
