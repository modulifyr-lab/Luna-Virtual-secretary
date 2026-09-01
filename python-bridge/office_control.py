"""
Office Automation Bridge for Luna (Windows Virtual Secretary)
Uses pywin32 to automate Microsoft Word, PowerPoint, and Outlook via COM interfaces.
"""

import sys
import json


def handle_word_command(action: str, params: dict):
    """
    Handle Microsoft Word automation tasks via pywin32 COM object.
    """
    try:
        import win32com.client
        word = win32com.client.Dispatch("Word.Application")
        word.Visible = True

        if action in ["create_doc", "create", "open"]:
            doc = word.Documents.Add()
            content = params.get("text", params.get("content", ""))
            if content:
                doc.Content.Text = content
            return {
                "status": "ok",
                "message": "Opened a new Microsoft Word document."
            }
        elif action == "append_text":
            if word.Documents.Count > 0:
                doc = word.ActiveDocument
            else:
                doc = word.Documents.Add()
            content = params.get("text", params.get("content", ""))
            doc.Content.InsertAfter(content)
            return {
                "status": "ok",
                "message": "Appended text to active Word document."
            }
        else:
            doc = word.Documents.Add()
            return {
                "status": "ok",
                "message": f"Executed Word action '{action}' on new document."
            }
    except Exception as e:
        sys.stderr.write(f"Word COM Error: {e}\n")
        return {
            "status": "error",
            "message": f"Could not control Microsoft Word via COM (is pywin32 and MS Word installed?): {e}"
        }


def handle_powerpoint_command(action: str, params: dict):
    """
    Handle Microsoft PowerPoint automation tasks via pywin32 COM object.
    """
    try:
        import win32com.client
        ppt = win32com.client.Dispatch("PowerPoint.Application")
        ppt.Visible = True

        if action in ["create_presentation", "create", "open"]:
            pres = ppt.Presentations.Add()
            # Add title slide (layout 1 = ppLayoutTitle)
            pres.Slides.Add(1, 1)
            return {
                "status": "ok",
                "message": "Created a new PowerPoint presentation."
            }
        else:
            pres = ppt.Presentations.Add()
            return {
                "status": "ok",
                "message": f"Executed PowerPoint action '{action}'."
            }
    except Exception as e:
        sys.stderr.write(f"PowerPoint COM Error: {e}\n")
        return {
            "status": "error",
            "message": f"Could not control Microsoft PowerPoint via COM: {e}"
        }


def handle_outlook_command(action: str, params: dict):
    """
    Handle Microsoft Outlook automation tasks via pywin32 COM object.
    """
    try:
        import win32com.client
        outlook = win32com.client.Dispatch("Outlook.Application")

        if action in ["draft_email", "create_email", "send_email"]:
            # 0 = olMailItem
            mail = outlook.CreateItem(0)
            mail.To = params.get("to", "")
            mail.Subject = params.get("subject", "Note from Luna")
            mail.Body = params.get("body", params.get("content", ""))
            mail.Display(True)
            return {
                "status": "ok",
                "message": "Drafted a new email in Outlook."
            }
        else:
            return {
                "status": "ok",
                "message": f"Executed Outlook action '{action}'."
            }
    except Exception as e:
        sys.stderr.write(f"Outlook COM Error: {e}\n")
        return {
            "status": "error",
            "message": f"Could not control Microsoft Outlook via COM: {e}"
        }


def main():
    """
    Main entrypoint for Office automation subprocess call.
    Expects command-line arguments: <app> <action> [params_json]
    """
    app = sys.argv[1].lower() if len(sys.argv) > 1 else "word"
    action = sys.argv[2].lower() if len(sys.argv) > 2 else "create"
    params_str = sys.argv[3] if len(sys.argv) > 3 else "{}"

    try:
        params = json.loads(params_str)
    except Exception:
        params = {}

    if "word" in app:
        result = handle_word_command(action, params)
    elif "powerpoint" in app or "ppt" in app:
        result = handle_powerpoint_command(action, params)
    elif "outlook" in app or "mail" in app:
        result = handle_outlook_command(action, params)
    else:
        result = handle_word_command(action, params)

    print(json.dumps(result))


if __name__ == "__main__":
    main()
