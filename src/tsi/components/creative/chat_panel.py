"""Chat panel component for creative workspace.

This module provides a chat interface connected to an AI assistant (Ollama)
to help users build proposals and understand scheduling concepts.
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING

import streamlit as st

if TYPE_CHECKING:
    pass

logger = logging.getLogger(__name__)

# Session state keys for chat
KEY_CHAT_MESSAGES = "creative_chat_messages"
KEY_CHAT_INPUT = "creative_chat_input"


def initialize_chat_state() -> None:
    """Initialize chat session state."""
    if KEY_CHAT_MESSAGES not in st.session_state:
        st.session_state[KEY_CHAT_MESSAGES] = [
            {
                "role": "assistant",
                "content": (
                    "👋 Welcome to the Creative Scheduling Workspace!\n\n"
                    "I can help you:\n"
                    "- Build observation proposals\n"
                    "- Configure scheduling parameters\n"
                    "- Understand scheduling concepts\n"
                    "- Run scheduling simulations\n\n"
                    "What would you like to create today?"
                ),
            }
        ]


def get_chat_messages() -> list[dict]:
    """Get current chat messages."""
    initialize_chat_state()
    return st.session_state[KEY_CHAT_MESSAGES]


def add_chat_message(role: str, content: str) -> None:
    """Add a message to chat history."""
    initialize_chat_state()
    st.session_state[KEY_CHAT_MESSAGES].append({"role": role, "content": content})


def clear_chat_history() -> None:
    """Clear chat history and reinitialize."""
    st.session_state[KEY_CHAT_MESSAGES] = []
    initialize_chat_state()


def render_chat_panel() -> None:
    """
    Render the chat panel for the creative workspace.
    
    The chat is connected to an Ollama-based chatbot for assistance
    with proposal building and scheduling concepts.
    """
    st.markdown("### 💬 Assistant")
    
    # Chat container with scrollable history
    chat_container = st.container(height=500)
    
    with chat_container:
        messages = get_chat_messages()
        for msg in messages:
            with st.chat_message(msg["role"]):
                st.markdown(msg["content"])
    
    # Chat input
    if prompt := st.chat_input("Ask about scheduling or proposals...", key="creative_chat_input"):
        # Add user message
        add_chat_message("user", prompt)
        
        # Generate response (placeholder for Ollama integration)
        response = _generate_response(prompt)
        add_chat_message("assistant", response)
        
        st.rerun()
    
    # Clear chat button
    col1, col2 = st.columns([3, 1])
    with col2:
        if st.button("🗑️ Clear", key="clear_chat_btn", width="stretch"):
            clear_chat_history()
            st.rerun()


def _generate_response(user_message: str) -> str:
    """
    Generate a response to the user message.
    
    TODO: Integrate with Ollama for AI-powered responses.
    
    OLLAMA INTEGRATION INSTRUCTIONS:
    ================================
    1. Install Ollama: https://ollama.ai/download
    
    2. Pull a model (recommended: llama3.1 or mistral):
       $ ollama pull llama3.1
    
    3. Start Ollama server:
       $ ollama serve
    
    4. Install Python client:
       $ pip install ollama
    
    5. Replace this placeholder with Ollama client:
    
       import ollama
       
       def _generate_response(user_message: str) -> str:
           # System prompt for scheduling assistant
           system_prompt = '''You are a telescope scheduling assistant.
           Help users build observation proposals and understand scheduling.
           Be concise and helpful. Use astronomical terminology appropriately.
           '''
           
           response = ollama.chat(
               model='llama3.1',
               messages=[
                   {'role': 'system', 'content': system_prompt},
                   *get_chat_messages(),
                   {'role': 'user', 'content': user_message}
               ]
           )
           return response['message']['content']
    
    6. For streaming responses, use:
    
       def _generate_response_stream(user_message: str):
           stream = ollama.chat(
               model='llama3.1',
               messages=[...],
               stream=True
           )
           for chunk in stream:
               yield chunk['message']['content']
    
    7. Environment configuration:
       Set OLLAMA_HOST if not running locally:
       $ export OLLAMA_HOST=http://your-ollama-server:11434
    
    Args:
        user_message: The user's input message.
    
    Returns:
        Generated response string.
    """
    # Placeholder responses based on keywords
    message_lower = user_message.lower()
    
    if any(word in message_lower for word in ["proposal", "create", "add", "new"]):
        return (
            "To create a new proposal, use the **Proposal Builder** panel on the right:\n\n"
            "1. Click **Add Task** to create observation tasks\n"
            "2. Set target coordinates (RA/Dec)\n"
            "3. Configure priority and duration\n"
            "4. Add constraints (altitude, azimuth limits)\n\n"
            "Would you like help with a specific target?"
        )
    
    if any(word in message_lower for word in ["schedule", "run", "execute", "simulate"]):
        return (
            "To run the scheduler:\n\n"
            "1. Configure algorithm in **Scheduler Config** panel\n"
            "2. Set the observation period (start/end dates)\n"
            "3. Configure location parameters\n"
            "4. Click **Run Scheduler** button\n\n"
            "The scheduler will process your proposals and show results!"
        )
    
    if any(word in message_lower for word in ["algorithm", "accumulative", "hybrid"]):
        return (
            "**Available Scheduling Algorithms:**\n\n"
            "- **Accumulative**: Default algorithm, builds schedule incrementally\n"
            "- **Hybrid Accumulative**: Multi-threaded variant for faster processing\n\n"
            "**Key Parameters:**\n"
            "- `max_iterations`: Maximum optimization iterations (0 = default)\n"
            "- `time_limit_seconds`: Time limit for scheduling (0 = no limit)\n"
            "- `seed`: Random seed for reproducibility (-1 = random)\n"
        )
    
    if any(word in message_lower for word in ["priority", "weight"]):
        return (
            "**Priority** determines observation importance:\n\n"
            "- Range: 0.0 (lowest) to 10.0 (highest)\n"
            "- Higher priority tasks are scheduled first\n"
            "- Equal priorities are resolved by visibility windows\n\n"
            "Tip: Use priorities strategically for critical observations!"
        )
    
    if any(word in message_lower for word in ["visibility", "window", "period"]):
        return (
            "**Visibility Windows** define when targets can be observed:\n\n"
            "- Computed based on target coordinates and constraints\n"
            "- Consider altitude limits, azimuth restrictions\n"
            "- Dark periods (no daylight observations)\n\n"
            "The scheduler automatically computes visibility for each task."
        )
    
    if any(word in message_lower for word in ["help", "what can", "how do"]):
        return (
            "I can help you with:\n\n"
            "📝 **Building Proposals**\n"
            "- Adding observation tasks\n"
            "- Setting target coordinates\n"
            "- Configuring constraints\n\n"
            "⚙️ **Scheduling**\n"
            "- Choosing algorithms\n"
            "- Setting parameters\n"
            "- Running simulations\n\n"
            "📊 **Analysis**\n"
            "- Understanding results\n"
            "- Optimizing schedules\n\n"
            "Just ask about any topic!"
        )
    
    # Default response
    return (
        "I'm here to help with telescope scheduling! You can ask about:\n"
        "- Creating observation proposals\n"
        "- Configuring the scheduler\n"
        "- Understanding scheduling concepts\n"
        "- Running simulations\n\n"
        "*Note: Full AI responses coming soon with Ollama integration.*"
    )
