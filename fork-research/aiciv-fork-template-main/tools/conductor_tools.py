#!/usr/bin/env python3
"""
Conductor Tools - Integrated helper for The Conductor

Provides easy functions to:
- Update Observatory state
- Send email reports
- Backup to GitHub

Import this module in agent workflows for automatic reporting.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

sys.path.insert(0, str(Path(__file__).parent.parent / ".claude/observatory"))

from observatory import (
    start_deployment,
    update_agent_status,
    log_agent_activity,
    complete_agent,
    complete_deployment,
    load_state
)
from tools.email_reporter import send_deployment_report, send_agent_update
from tools.github_backup import auto_backup


class Mission:
    """
    Helper class for managing missions with automatic reporting

    Usage:
        mission = Mission("Analyze authentication system")
        mission.add_agent("code-archaeologist")
        mission.add_agent("security-auditor")

        mission.start()

        # Update agent progress
        mission.update_agent("code-archaeologist", "working", 50, "Tracing JWT flow")

        # Complete agent
        mission.complete_agent("code-archaeologist", [
            "Uses JWT-based auth",
            "Tokens stored in Redis"
        ])

        # Complete mission
        mission.complete("All authentication flows documented")
    """

    def __init__(self, task_description, email_updates=True, github_backup=True):
        self.task = task_description
        self.agents = []
        self.deployment_id = None
        self.email_updates = email_updates
        self.github_backup = github_backup

    def add_agent(self, agent_name):
        """Add agent to mission"""
        self.agents.append(agent_name)

    def start(self):
        """Start mission - initializes Observatory tracking"""
        print(f"🎭 Mission started: {self.task}")
        print(f"🤖 Agents: {', '.join(self.agents)}")

        self.deployment_id = start_deployment(self.task, self.agents)
        print(f"📊 Tracking deployment: {self.deployment_id}")

        return self.deployment_id

    def update_agent(self, agent_name, status, progress, activity):
        """Update agent status in Observatory"""
        update_agent_status(agent_name, status, progress, activity)
        print(f"   {agent_name}: {progress}% - {activity}")

    def log_activity(self, agent_name, message):
        """Log agent activity"""
        log_agent_activity(agent_name, message)

    def complete_agent(self, agent_name, findings):
        """Mark agent complete and optionally send email update"""
        complete_agent(agent_name, findings)
        print(f"✅ {agent_name} completed with {len(findings)} findings")

        # Send email update for important completions
        if self.email_updates and len(findings) > 0:
            send_agent_update(agent_name, "completed", "Analysis complete", findings)

    def complete(self, synthesis):
        """Complete mission - sends report and backs up to GitHub"""
        print(f"\n🎯 Mission complete: {self.task}")

        # Complete in Observatory
        complete_deployment(self.deployment_id, synthesis)

        # Load final deployment data for reporting
        state = load_state()
        completed_deployment = None

        for dep in state.get('history', []):
            if dep.get('id') == self.deployment_id:
                completed_deployment = dep
                break

        # Send email report
        if self.email_updates and completed_deployment:
            print("📧 Sending email report...")
            send_deployment_report(completed_deployment)

        # Backup to GitHub
        if self.github_backup:
            print("📦 Backing up to GitHub...")
            auto_backup(f"Mission complete: {self.task}")

        print("✨ Mission complete! All systems updated.")


def quick_mission(task, agents, synthesis, findings_per_agent=None):
    """
    Quick mission helper - for simple missions

    Usage:
        quick_mission(
            "Analyze API endpoints",
            ["api-architect", "security-auditor"],
            "Found 5 endpoints, 2 need authentication",
            {
                "api-architect": ["REST API with 5 endpoints", "Good documentation"],
                "security-auditor": ["Missing rate limiting", "No input validation"]
            }
        )
    """
    mission = Mission(task)

    for agent in agents:
        mission.add_agent(agent)

    mission.start()

    # Simulate completion
    if findings_per_agent:
        for agent_name, findings in findings_per_agent.items():
            mission.update_agent(agent_name, "working", 50, "Analyzing...")
            mission.update_agent(agent_name, "working", 100, "Complete")
            mission.complete_agent(agent_name, findings)
    else:
        # No specific findings, just mark complete
        for agent in agents:
            mission.update_agent(agent, "completed", 100, "Analysis complete")

    mission.complete(synthesis)

    return mission.deployment_id


# Example usage
if __name__ == '__main__':
    print("🎭 Conductor Tools - Mission Management Demo")
    print("=" * 60)

    # Example 1: Full mission workflow
    print("\n📋 Example 1: Full Mission Workflow")
    mission = Mission("Test mission for conductor tools")
    mission.add_agent("code-archaeologist")
    mission.add_agent("pattern-detector")

    mission.start()

    # Simulate agent work
    mission.update_agent("code-archaeologist", "working", 25, "Analyzing file structure")
    mission.log_activity("code-archaeologist", "Found 50 source files")
    mission.update_agent("code-archaeologist", "working", 75, "Mapping dependencies")
    mission.complete_agent("code-archaeologist", [
        "System uses 4-layer architecture",
        "14 specialized agents defined"
    ])

    mission.update_agent("pattern-detector", "working", 50, "Identifying patterns")
    mission.complete_agent("pattern-detector", [
        "Consistent agent structure detected",
        "Pattern quality: 10/10"
    ])

    mission.complete("Test mission completed successfully with automated reporting!")

    print("\n✅ Demo complete! Check:")
    print("   • Observatory dashboard: http://localhost:5000")
    print("   • Email: ${HUMAN_NAME_LOWER}cmusic@gmail.com")
    print("   • GitHub: https://github.com/ai-CIV-2025/ai-civ-collective")
