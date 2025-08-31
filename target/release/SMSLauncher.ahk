; TCast SMS Gateway Automation Script - AutoHotkey v2
; Press F1 to start the automation
; Press Escape to exit the script

#Requires AutoHotkey v2.0
#SingleInstance Force

; Global variables
global counter := 17  ; Starting counter value
global maxCounter := 29  ; Maximum counter value before reset
global appPath := "C:\Program Files (x86)\TCast SMS Gateway V4\TCast GSM Gateway V4.exe"

; Hotkey to start automation (F1)
F1::RunAutomation()

; Hotkey to exit script (Escape)
Esc::ExitApp

; Main automation function
RunAutomation() {
    global counter, maxCounter, appPath
    
    ; Ask user if they want to repeat the operation
    result := MsgBox("Do you want to run the automation?`nPress Yes to continue, No to cancel.", "TCast Automation", "YesNo")
    if (result = "No")
        return
    
    ; Check if application exists
    if (!FileExist(appPath)) {
        MsgBox("Application not found at:`n" . appPath, "Error", "OK Icon!")
        return
    }
    
    ; Run the application
    try {
        Run(appPath)
    } catch {
        MsgBox("Failed to run the application.", "Error", "OK Icon!")
        return
    }
    
    ; Wait for application to load (2 seconds as per original)
    Sleep(2000)
    
    ; Start the main loop
    Loop {
        ; Perform the automation sequence
        PerformSequence()
        
        ; Ask if user wants to repeat
        result := MsgBox("Operation completed. Counter is now: " . counter . "`nDo you want to repeat?", "Continue?", "YesNo")
        if (result = "No")
            break
    }
}

; Main sequence function
PerformSequence() {
    global counter, maxCounter
    
    ; Small delay before starting
    Sleep(100)
    
    ; Alt+F (File menu)
    Send("!f")
    Sleep(100)
    
    ; S (likely "Save" or similar option)
    Send("s")
    Sleep(200)
    
    ; Navigate with Tab (4 times)
    Loop 4 {
        Send("{Tab}")
        Sleep(100)
    }
    
    ; Select text with Ctrl+Shift+Right
    Send("^+{Right}")
    Sleep(100)
    
    ; Copy selected text
    Send("^c")
    Sleep(100)
    
    ; Delete selected text
    Send("{BackSpace}")
    Sleep(100)
    
    ; Check clipboard and manage counter
    clipboardValue := A_Clipboard
    if (IsNumber(clipboardValue)) {
        currentValue := Number(clipboardValue)
        if (currentValue >= maxCounter) {
            counter := 17  ; Reset to starting value
        }
    }
    
    ; Set clipboard to counter value
    A_Clipboard := String(counter)
    Sleep(100)
    
    ; Paste the counter value
    Send("^v")
    Sleep(100)
    
    ; Increment counter for next iteration
    counter++
    
    ; Tab and Enter twice
    Send("{Tab}")
    Sleep(100)
    Send("{Enter}")
    Sleep(100)
    Send("{Enter}")
    Sleep(100)
    
    ; Alt+E menu
    Send("!e")
    Sleep(100)
    
    ; S option
    Send("s")
    Sleep(100)
    
    ; Tab and Down arrow
    Send("{Tab}")
    Sleep(100)
    Send("{Down}")
    Sleep(100)
    
    ; Alt+E again
    Send("!e")
    Sleep(100)
    
    ; T option
    Send("t")
    Sleep(1250)  ; Longer delay here as per original
    
    ; Alt+E once more
    Send("!e")
    Sleep(100)
    
    ; Up arrow and Enter
    Send("{Up}")
    Sleep(100)
    Send("{Enter}")
    Sleep(100)
}

; Helper function to check if a string is a number
IsNumber(str) {
    if (str = "")
        return false
    Loop Parse, str {
        if (!IsDigit(A_LoopField) && A_LoopField != "-" && A_LoopField != ".")
            return false
    }
    return true
}