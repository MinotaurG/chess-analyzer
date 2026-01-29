// Coordinate Trainer - Full Implementation

(function() {
    'use strict';

    // ==========================================================================
    // STATE
    // ==========================================================================
    
    const state = {
        mode: 'find',           // 'find', 'name', 'color'
        perspective: 'white',   // 'white', 'black'
        timer: 0,               // 0 = off, or seconds
        showCoords: false,
        
        currentSquare: null,
        startTime: 0,
        
        attempts: 0,
        correct: 0,
        times: [],
        history: [],            // {square, correct, time}
        
        waiting: false,         // waiting for next question
    };

    // ==========================================================================
    // AUDIO
    // ==========================================================================
    
    const audioCtx = new (window.AudioContext || window.webkitAudioContext)();
    
    function playSound(correct) {
        const osc = audioCtx.createOscillator();
        const gain = audioCtx.createGain();
        
        osc.connect(gain);
        gain.connect(audioCtx.destination);
        
        osc.frequency.value = correct ? 880 : 220;
        osc.type = correct ? 'sine' : 'square';
        
        gain.gain.setValueAtTime(0.1, audioCtx.currentTime);
        gain.gain.exponentialRampToValueAtTime(0.01, audioCtx.currentTime + 0.2);
        
        osc.start(audioCtx.currentTime);
        osc.stop(audioCtx.currentTime + 0.2);
    }

    // ==========================================================================
    // BOARD
    // ==========================================================================
    
    const FILES = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
    const RANKS = ['1', '2', '3', '4', '5', '6', '7', '8'];
    
    function isLightSquare(file, rank) {
        const fileIdx = FILES.indexOf(file);
        const rankIdx = parseInt(rank);
        return (fileIdx + rankIdx) % 2 === 1;
    }
    
    function randomSquare() {
        const file = FILES[Math.floor(Math.random() * 8)];
        const rank = RANKS[Math.floor(Math.random() * 8)];
        return file + rank;
    }
    
    function renderBoard() {
        const board = document.getElementById('chess-board');
        if (!board) return;
        
        board.innerHTML = '';
        
        const ranks = state.perspective === 'white' 
            ? [...RANKS].reverse() 
            : RANKS;
        const files = state.perspective === 'white' 
            ? FILES 
            : [...FILES].reverse();
        
        for (const rank of ranks) {
            for (const file of files) {
                const square = file + rank;
                const isLight = isLightSquare(file, rank);
                
                const div = document.createElement('div');
                div.className = `chess-square ${isLight ? 'light' : 'dark'}`;
                div.dataset.square = square;
                
                // Add coordinates on edge squares
                if (state.showCoords || 
                    (state.perspective === 'white' && rank === '1') ||
                    (state.perspective === 'black' && rank === '8')) {
                    if ((state.perspective === 'white' && rank === '1') ||
                        (state.perspective === 'black' && rank === '8')) {
                        const coord = document.createElement('span');
                        coord.className = 'coord coord-file';
                        coord.textContent = file;
                        div.appendChild(coord);
                    }
                }
                
                if ((state.perspective === 'white' && file === 'a') ||
                    (state.perspective === 'black' && file === 'h')) {
                    const coord = document.createElement('span');
                    coord.className = 'coord coord-rank';
                    coord.textContent = rank;
                    div.appendChild(coord);
                }
                
                // Click handler for 'find' mode
                div.addEventListener('click', () => handleSquareClick(square));
                
                board.appendChild(div);
            }
        }
        
        // Show target dot in 'name' mode
        if (state.mode === 'name' && state.currentSquare) {
            const targetEl = board.querySelector(`[data-square="${state.currentSquare}"]`);
            if (targetEl) {
                const dot = document.createElement('div');
                dot.className = 'target-dot';
                targetEl.appendChild(dot);
            }
        }
    }
    
    function highlightSquare(square, className) {
        const el = document.querySelector(`[data-square="${square}"]`);
        if (el) {
            el.classList.add(className);
        }
    }
    
    function clearHighlights() {
        document.querySelectorAll('.chess-square').forEach(el => {
            el.classList.remove('correct', 'wrong', 'target');
        });
    }

    // ==========================================================================
    // GAME LOGIC
    // ==========================================================================
    
    function newQuestion() {
        state.waiting = false;
        state.currentSquare = randomSquare();
        state.startTime = Date.now();
        
        clearHighlights();
        renderBoard();
        updatePrompt();
        
        // Focus input in 'name' mode
        if (state.mode === 'name') {
            const input = document.getElementById('square-input');
            if (input) {
                input.value = '';
                input.focus();
            }
        }
        
        // Start timer if enabled
        if (state.timer > 0) {
            startTimer();
        }
    }
    
    function handleAnswer(answer, clicked = null) {
        if (state.waiting) return;
        
        const elapsed = Date.now() - state.startTime;
        let isCorrect = false;
        
        if (state.mode === 'find') {
            isCorrect = answer === state.currentSquare;
        } else if (state.mode === 'name') {
            isCorrect = answer.toLowerCase().trim() === state.currentSquare;
        } else if (state.mode === 'color') {
            const shouldBeLight = isLightSquare(
                state.currentSquare[0], 
                state.currentSquare[1]
            );
            isCorrect = (answer === 'light') === shouldBeLight;
        }
        
        // Update state
        state.attempts++;
        if (isCorrect) state.correct++;
        state.times.push(elapsed);
        state.history.push({
            square: state.currentSquare,
            correct: isCorrect,
            time: elapsed
        });
        
        // Visual feedback
        playSound(isCorrect);
        
        if (state.mode === 'find') {
            if (isCorrect) {
                highlightSquare(state.currentSquare, 'correct');
            } else {
                if (clicked) highlightSquare(clicked, 'wrong');
                highlightSquare(state.currentSquare, 'correct');
            }
        } else if (state.mode === 'name') {
            highlightSquare(state.currentSquare, isCorrect ? 'correct' : 'wrong');
        } else if (state.mode === 'color') {
            highlightSquare(state.currentSquare, isCorrect ? 'correct' : 'wrong');
        }
        
        showFeedback(isCorrect, elapsed);
        updateStats();
        
        // Next question
        state.waiting = true;
        setTimeout(newQuestion, isCorrect ? 500 : 1200);
    }
    
    function handleSquareClick(square) {
        if (state.mode !== 'find') return;
        if (state.waiting) return;
        
        handleAnswer(square, square);
    }
    
    function handleColorAnswer(color) {
        if (state.mode !== 'color') return;
        if (state.waiting) return;
        
        handleAnswer(color);
    }

    // ==========================================================================
    // TIMER
    // ==========================================================================
    
    let timerInterval = null;
    
    function startTimer() {
        stopTimer();
        
        const duration = state.timer * 1000;
        const startTime = Date.now();
        const timerFill = document.getElementById('timer-fill');
        
        if (!timerFill) return;
        
        timerFill.style.width = '100%';
        timerFill.classList.remove('warning', 'danger');
        
        timerInterval = setInterval(() => {
            const elapsed = Date.now() - startTime;
            const remaining = Math.max(0, duration - elapsed);
            const percent = (remaining / duration) * 100;
            
            timerFill.style.width = percent + '%';
            
            if (percent < 20) {
                timerFill.classList.add('danger');
            } else if (percent < 40) {
                timerFill.classList.add('warning');
            }
            
            if (remaining <= 0) {
                stopTimer();
                handleAnswer('__timeout__');
            }
        }, 50);
    }
    
    function stopTimer() {
        if (timerInterval) {
            clearInterval(timerInterval);
            timerInterval = null;
        }
    }

    // ==========================================================================
    // UI UPDATES
    // ==========================================================================
    
    function updatePrompt() {
        const promptText = document.getElementById('prompt-text');
        const promptTarget = document.getElementById('prompt-target');
        const promptInput = document.getElementById('prompt-input');
        const colorButtons = document.getElementById('color-buttons');
        
        if (state.mode === 'find') {
            if (promptText) promptText.textContent = 'Click on:';
            if (promptTarget) promptTarget.textContent = state.currentSquare;
            if (promptInput) promptInput.style.display = 'none';
            if (colorButtons) colorButtons.style.display = 'none';
        } else if (state.mode === 'name') {
            if (promptText) promptText.textContent = 'Name this square:';
            if (promptTarget) promptTarget.textContent = '';
            if (promptInput) promptInput.style.display = 'block';
            if (colorButtons) colorButtons.style.display = 'none';
        } else if (state.mode === 'color') {
            if (promptText) promptText.textContent = 'What color is:';
            if (promptTarget) promptTarget.textContent = state.currentSquare;
            if (promptInput) promptInput.style.display = 'none';
            if (colorButtons) colorButtons.style.display = 'flex';
        }
    }
    
    function showFeedback(correct, timeMs) {
        const feedback = document.getElementById('feedback');
        if (!feedback) return;
        
        if (correct) {
            feedback.textContent = `Correct! (${(timeMs/1000).toFixed(2)}s)`;
            feedback.className = 'feedback correct';
        } else {
            feedback.textContent = `Wrong - it was ${state.currentSquare}`;
            feedback.className = 'feedback wrong';
        }
    }
    
    function updateStats() {
        const attemptsEl = document.getElementById('stat-attempts');
        const accuracyEl = document.getElementById('stat-accuracy');
        const avgTimeEl = document.getElementById('stat-avgtime');
        const bestTimeEl = document.getElementById('stat-besttime');
        
        if (attemptsEl) attemptsEl.textContent = state.attempts;
        
        if (accuracyEl) {
            const pct = state.attempts > 0 
                ? ((state.correct / state.attempts) * 100).toFixed(1) 
                : '0.0';
            accuracyEl.textContent = pct + '%';
        }
        
        if (avgTimeEl) {
            const avg = state.times.length > 0
                ? (state.times.reduce((a,b) => a+b, 0) / state.times.length / 1000).toFixed(2)
                : '0.00';
            avgTimeEl.textContent = avg + 's';
        }
        
        if (bestTimeEl) {
            const correctTimes = state.history
                .filter(h => h.correct)
                .map(h => h.time);
            const best = correctTimes.length > 0
                ? (Math.min(...correctTimes) / 1000).toFixed(2)
                : '-';
            bestTimeEl.textContent = best === '-' ? '-' : best + 's';
        }
        
        updateWeakSquares();
    }
    
    function updateWeakSquares() {
        const container = document.getElementById('weak-squares');
        if (!container) return;
        
        // Calculate per-square stats
        const stats = {};
        for (const h of state.history) {
            if (!stats[h.square]) {
                stats[h.square] = { total: 0, correct: 0, time: 0 };
            }
            stats[h.square].total++;
            if (h.correct) stats[h.square].correct++;
            stats[h.square].time += h.time;
        }
        
        // Find weak squares (< 80% accuracy or slow)
        const avgTime = state.times.length > 0
            ? state.times.reduce((a,b) => a+b, 0) / state.times.length
            : 2000;
        
        const weak = Object.entries(stats)
            .map(([sq, s]) => ({
                square: sq,
                accuracy: (s.correct / s.total) * 100,
                avgTime: s.time / s.total
            }))
            .filter(s => s.accuracy < 80 || s.avgTime > avgTime * 1.5)
            .sort((a, b) => a.accuracy - b.accuracy)
            .slice(0, 5);
        
        if (weak.length === 0) {
            container.innerHTML = '<div class="weak-item" style="color: var(--text-muted)">No weak squares yet</div>';
            return;
        }
        
        container.innerHTML = weak.map(w => `
            <div class="weak-item">
                <span class="weak-square">${w.square}</span>
                <span class="weak-stats">${w.accuracy.toFixed(0)}% / ${(w.avgTime/1000).toFixed(1)}s</span>
            </div>
        `).join('');
    }
    
    function updateModeButtons() {
        document.querySelectorAll('[data-mode]').forEach(btn => {
            btn.classList.toggle('active', btn.dataset.mode === state.mode);
        });
    }
    
    function updatePerspectiveButtons() {
        document.querySelectorAll('[data-perspective]').forEach(btn => {
            btn.classList.toggle('active', btn.dataset.perspective === state.perspective);
        });
    }
    
    function updateTimerButtons() {
        document.querySelectorAll('[data-timer]').forEach(btn => {
            btn.classList.toggle('active', parseInt(btn.dataset.timer) === state.timer);
        });
    }

    // ==========================================================================
    // MODE / SETTINGS
    // ==========================================================================
    
    function setMode(mode) {
        state.mode = mode;
        updateModeButtons();
        newQuestion();
    }
    
    function setPerspective(perspective) {
        state.perspective = perspective;
        updatePerspectiveButtons();
        renderBoard();
    }
    
    function setTimer(seconds) {
        state.timer = seconds;
        updateTimerButtons();
        stopTimer();
        
        const timerBar = document.getElementById('timer-bar');
        if (timerBar) {
            timerBar.style.display = seconds > 0 ? 'block' : 'none';
        }
    }
    
    function resetSession() {
        state.attempts = 0;
        state.correct = 0;
        state.times = [];
        state.history = [];
        updateStats();
        newQuestion();
    }

    // ==========================================================================
    // KEYBOARD
    // ==========================================================================
    
    function handleKeydown(e) {
        // Ignore if typing in input
        if (e.target.tagName === 'INPUT') {
            if (e.key === 'Enter') {
                const value = e.target.value;
                if (value.length >= 2) {
                    handleAnswer(value);
                }
            }
            return;
        }
        
        switch (e.key) {
            case ' ':
            case 'Space':
                e.preventDefault();
                if (state.waiting) {
                    // Skip wait, go to next
                    state.waiting = false;
                    newQuestion();
                }
                break;
            case 'r':
            case 'R':
                resetSession();
                break;
            case '1':
                setMode('find');
                break;
            case '2':
                setMode('name');
                break;
            case '3':
                setMode('color');
                break;
            case 'w':
                setPerspective('white');
                break;
            case 'b':
                setPerspective('black');
                break;
        }
    }

    // ==========================================================================
    // INIT
    // ==========================================================================
    
    function init() {
        // Bind mode buttons
        document.querySelectorAll('[data-mode]').forEach(btn => {
            btn.addEventListener('click', () => setMode(btn.dataset.mode));
        });
        
        // Bind perspective buttons
        document.querySelectorAll('[data-perspective]').forEach(btn => {
            btn.addEventListener('click', () => setPerspective(btn.dataset.perspective));
        });
        
        // Bind timer buttons
        document.querySelectorAll('[data-timer]').forEach(btn => {
            btn.addEventListener('click', () => setTimer(parseInt(btn.dataset.timer)));
        });
        
        // Bind reset button
        const resetBtn = document.getElementById('btn-reset');
        if (resetBtn) {
            resetBtn.addEventListener('click', resetSession);
        }
        
        // Bind color buttons
        document.querySelectorAll('[data-color]').forEach(btn => {
            btn.addEventListener('click', () => handleColorAnswer(btn.dataset.color));
        });
        
        // Keyboard
        document.addEventListener('keydown', handleKeydown);
        
        // Initial state
        updateModeButtons();
        updatePerspectiveButtons();
        updateTimerButtons();
        updateStats();
        newQuestion();
    }
    
    // Run on DOM ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

    // ==========================================================================
    // SAVE SESSION TO SERVER
    // ==========================================================================

    function saveSession() {
        if (state.attempts === 0) return;
        
        const totalTime = state.times.reduce((a, b) => a + b, 0);
        const correctTimes = state.history.filter(h => h.correct).map(h => h.time);
        const bestTime = correctTimes.length > 0 ? Math.min(...correctTimes) : null;
        
        fetch('/api/training/save', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                training_type: 'coordinates',
                attempts: state.attempts,
                correct: state.correct,
                total_time_ms: totalTime,
                best_time_ms: bestTime,
            }),
        }).catch(err => console.error('Failed to save session:', err));
    }

    // Save when leaving page
    window.addEventListener('beforeunload', saveSession);

    // Save periodically (every 30 seconds if there's activity)
    let lastSavedAttempts = 0;
    setInterval(() => {
        if (state.attempts > lastSavedAttempts) {
            saveSession();
            lastSavedAttempts = state.attempts;
        }
    }, 30000);
})();
