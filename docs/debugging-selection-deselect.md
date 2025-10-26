# Debugging Selection Deselection Issue

## Expected Behavior

### Single Click (no Shift)
- **Click on point**: Clear all other selections, select clicked point
- **Click on empty space**: Clear all selections (deselect everything)

### Shift + Click (additive)
- **Shift+click on point**: Add point to existing selection
- **Shift+click on already-selected point**: Toggle it off
- **Shift+click on empty space**: Keep existing selection

### Marquee (drag)
- **Drag without Shift**: Clear existing, select all in rectangle
- **Drag with Shift**: Keep existing, add all in rectangle

## Debug Logging

The code now has clear debug markers you can search for in the logs:

```bash
# View logs in real-time
tail -f ~/.config/bezy/logs/bezy.log

# Search for specific events
grep "🔵 EMPTY SPACE" ~/.config/bezy/logs/bezy.log
grep "🟢 POINT CLICKED" ~/.config/bezy/logs/bezy.log
```

### What to Look For

**When clicking on empty space:**
- Should see: `🔵 EMPTY SPACE CLICK - No point within selection margin`
- Should see: `🔵 CLEARED X selections`
- If you DON'T see these messages, a point is being found within the selection margin

**When clicking on a point:**
- Should see: `🟢 POINT CLICKED - Entity: ...`
- Should see selection being cleared if shift not held

## Common Issues

### Issue 1: Selection Margin Too Large
**Symptom**: Never see "EMPTY SPACE CLICK" message
**Cause**: `zoom_aware_margin` is too large, always finding a point
**Check**: Look for "Using selection margin: X (zoom-aware)" in logs
**Fix**: The margin scales with camera zoom - at zoom out it gets larger

### Issue 2: Camera Scale Wrong
**Symptom**: Margin doesn't change with zoom
**Cause**: Camera scale not being retrieved correctly
**Check**: Add debug for camera_scale value
**Fix**: Verify OrthographicProjection.scale is correct

### Issue 3: Wrong Modifier State
**Symptom**: Clears when shift is held, or doesn't clear when shift not held
**Cause**: Modifier state not being read correctly
**Check**: Look for "Shift held: true/false" in logs
**Fix**: Verify InputState.keyboard.modifiers.shift

## Code Locations

**Main selection click handler:**
- File: `src/editing/selection/input/mouse.rs`
- Function: `handle_selection_click()` (line ~607)
- Empty space logic: Line ~811
- Point found logic: Line ~713

**Zoom-aware margin calculation:**
- File: `src/systems/input_consumer.rs`
- Function: `process_selection_events()` (line ~754)
- Calculation: Line ~779

**Selection margin constant:**
- File: `src/editing/selection/events.rs`
- Constant: `SELECTION_MARGIN = 16.0`

## Testing Steps

1. **Open the app with a glyph**
2. **Click on a point** - should see green "POINT CLICKED" in logs
3. **Click far from any point** - should see blue "EMPTY SPACE CLICK" in logs
4. **Verify deselection** - should see "CLEARED X selections"

If step 3 never shows "EMPTY SPACE CLICK", the margin is too large.
