// Puzzle Trainer - Lichess-style

(function() {
    'use strict';

    // ==========================================================================
    // STATE
    // ==========================================================================
    
    const state = {
        puzzle: null,
        selectedSquare: null,
        solved: false,
        perspective: 'white',
        pieces: {},
        turn: 'white',
        lastMove: null,
        castling: '',
        
        // Drag state
        dragging: null,
        dragElement: null,
        dragStartX: 0,
        dragStartY: 0,
        hasDragged: false,
        currentHoverSquare: null,
        
        // Stats
        totalSolved: 0,
        totalAttempts: 0,
        
        // Animation lock
        animating: false,
    };

    const FILES = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
    const RANKS = ['1', '2', '3', '4', '5', '6', '7', '8'];
    const DRAG_THRESHOLD = 3;

    // ==========================================================================
    // AUDIO
    // ==========================================================================
    
    const sounds = {
        move: new Audio('/static/sounds/move.mp3'),
        capture: new Audio('/static/sounds/capture.mp3'),
        correct: new Audio('/static/sounds/correct.mp3'),
        wrong: new Audio('/static/sounds/wrong.mp3'),
    };
    
    Object.values(sounds).forEach(s => s.load());
    
    function playSound(type) {
        const sound = sounds[type];
        if (sound) {
            sound.currentTime = 0;
            sound.play().catch(() => {});
        }
    }

    // ==========================================================================
    // FEN PARSING
    // ==========================================================================

    function parseFen(fen) {
        const pieces = {};
        const parts = fen.split(' ');
        const position = parts[0];
        const turn = parts[1] || 'w';
        const castling = parts[2] || '-';
        const rows = position.split('/');
        
        for (let rankIdx = 0; rankIdx < 8; rankIdx++) {
            const row = rows[rankIdx];
            const rank = RANKS[7 - rankIdx];
            let fileIdx = 0;
            
            for (const char of row) {
                if (char >= '1' && char <= '8') {
                    fileIdx += parseInt(char);
                } else {
                    const file = FILES[fileIdx];
                    const square = file + rank;
                    pieces[square] = char;
                    fileIdx++;
                }
            }
        }
        
        return { 
            pieces, 
            turn: turn === 'w' ? 'white' : 'black',
            castling: castling === '-' ? '' : castling
        };
    }

    function getPieceImage(piece) {
        const color = piece === piece.toUpperCase() ? 'w' : 'b';
        const type = piece.toUpperCase();
        return `/static/pieces/${color}${type}.svg`;
    }

    function isOwnPiece(square) {
        const piece = state.pieces[square];
        if (!piece) return false;
        const isWhitePiece = piece === piece.toUpperCase();
        return (state.turn === 'white') === isWhitePiece;
    }

    function isEnemyPiece(square) {
        const piece = state.pieces[square];
        if (!piece) return false;
        const isWhitePiece = piece === piece.toUpperCase();
        return (state.turn === 'white') !== isWhitePiece;
    }

    // ==========================================================================
    // MOVE GENERATION
    // ==========================================================================

    function getPseudoLegalMoves(square) {
        const piece = state.pieces[square];
        if (!piece) return [];
        
        const moves = [];
        const file = square[0];
        const rank = square[1];
        const fileIdx = FILES.indexOf(file);
        const rankIdx = RANKS.indexOf(rank);
        const isWhite = piece === piece.toUpperCase();
        const type = piece.toUpperCase();
        
        const addMove = (f, r) => {
            if (f >= 0 && f < 8 && r >= 0 && r < 8) {
                const target = FILES[f] + RANKS[r];
                const targetPiece = state.pieces[target];
                if (!targetPiece || (isWhite !== (targetPiece === targetPiece.toUpperCase()))) {
                    moves.push(target);
                }
            }
        };
        
        const isEmpty = (sq) => !state.pieces[sq];
        
        const addSlide = (df, dr) => {
            let f = fileIdx + df;
            let r = rankIdx + dr;
            while (f >= 0 && f < 8 && r >= 0 && r < 8) {
                const target = FILES[f] + RANKS[r];
                const targetPiece = state.pieces[target];
                if (!targetPiece) {
                    moves.push(target);
                } else {
                    if (isWhite !== (targetPiece === targetPiece.toUpperCase())) {
                        moves.push(target);
                    }
                    break;
                }
                f += df;
                r += dr;
            }
        };
        
        switch (type) {
            case 'P':
                const dir = isWhite ? 1 : -1;
                const startRank = isWhite ? 1 : 6;
                const fwdRank = rankIdx + dir;
                if (fwdRank >= 0 && fwdRank < 8) {
                    const fwd = FILES[fileIdx] + RANKS[fwdRank];
                    if (!state.pieces[fwd]) {
                        moves.push(fwd);
                        if (rankIdx === startRank) {
                            const fwd2 = FILES[fileIdx] + RANKS[rankIdx + 2*dir];
                            if (!state.pieces[fwd2]) moves.push(fwd2);
                        }
                    }
                    for (const df of [-1, 1]) {
                        if (fileIdx + df >= 0 && fileIdx + df < 8) {
                            const cap = FILES[fileIdx + df] + RANKS[fwdRank];
                            if (state.pieces[cap] && isWhite !== (state.pieces[cap] === state.pieces[cap].toUpperCase())) {
                                moves.push(cap);
                            }
                        }
                    }
                }
                break;
            case 'N':
                for (const [df, dr] of [[-2,-1],[-2,1],[-1,-2],[-1,2],[1,-2],[1,2],[2,-1],[2,1]]) {
                    addMove(fileIdx + df, rankIdx + dr);
                }
                break;
            case 'B':
                for (const [df, dr] of [[-1,-1],[-1,1],[1,-1],[1,1]]) addSlide(df, dr);
                break;
            case 'R':
                for (const [df, dr] of [[-1,0],[1,0],[0,-1],[0,1]]) addSlide(df, dr);
                break;
            case 'Q':
                for (const [df, dr] of [[-1,-1],[-1,1],[1,-1],[1,1],[-1,0],[1,0],[0,-1],[0,1]]) addSlide(df, dr);
                break;
            case 'K':
                for (const [df, dr] of [[-1,-1],[-1,0],[-1,1],[0,-1],[0,1],[1,-1],[1,0],[1,1]]) {
                    addMove(fileIdx + df, rankIdx + dr);
                }
                // Castling
                if (isWhite && rank === '1' && file === 'e') {
                    if (state.castling.includes('K') && isEmpty('f1') && isEmpty('g1')) {
                        moves.push('g1');
                    }
                    if (state.castling.includes('Q') && isEmpty('d1') && isEmpty('c1') && isEmpty('b1')) {
                        moves.push('c1');
                    }
                } else if (!isWhite && rank === '8' && file === 'e') {
                    if (state.castling.includes('k') && isEmpty('f8') && isEmpty('g8')) {
                        moves.push('g8');
                    }
                    if (state.castling.includes('q') && isEmpty('d8') && isEmpty('c8') && isEmpty('b8')) {
                        moves.push('c8');
                    }
                }
                break;
        }
        
        return moves;
    }

    // ==========================================================================
    // BOARD RENDERING
    // ==========================================================================

    function isLightSquare(file, rank) {
        const fileIdx = FILES.indexOf(file);
        const rankIdx = parseInt(rank);
        return (fileIdx + rankIdx) % 2 === 1;
    }

    function getSquareElement(square) {
        return document.querySelector(`[data-square="${square}"]`);
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
        
        const validMoves = state.selectedSquare ? getPseudoLegalMoves(state.selectedSquare) : [];
        
        for (const rank of ranks) {
            for (const file of files) {
                const square = file + rank;
                const isLight = isLightSquare(file, rank);
                
                const div = document.createElement('div');
                div.className = `chess-square ${isLight ? 'light' : 'dark'}`;
                div.dataset.square = square;
                
                if (state.selectedSquare === square) {
                    div.classList.add('selected');
                }
                
                if (state.lastMove && (state.lastMove.from === square || state.lastMove.to === square)) {
                    div.classList.add('last-move');
                }
                
                const piece = state.pieces[square];
                if (piece) {
                    const img = document.createElement('img');
                    img.src = getPieceImage(piece);
                    img.className = 'piece';
                    img.draggable = false;
                    img.dataset.square = square;
                    div.appendChild(img);
                    
                    if (validMoves.includes(square) && isEnemyPiece(square)) {
                        div.classList.add('can-capture');
                    }
                }
                
                if (validMoves.includes(square) && !piece) {
                    const dot = document.createElement('div');
                    dot.className = 'move-dot';
                    div.appendChild(dot);
                }
                
                if (rank === (state.perspective === 'white' ? '1' : '8')) {
                    const coord = document.createElement('span');
                    coord.className = 'coord coord-file';
                    coord.textContent = file;
                    div.appendChild(coord);
                }
                if (file === (state.perspective === 'white' ? 'a' : 'h')) {
                    const coord = document.createElement('span');
                    coord.className = 'coord coord-rank';
                    coord.textContent = rank;
                    div.appendChild(coord);
                }
                
                board.appendChild(div);
            }
        }
        
        const turnIndicator = document.getElementById('turn-indicator');
        if (turnIndicator) {
            turnIndicator.textContent = `${state.turn === 'white' ? 'White' : 'Black'} to move`;
        }
    }

    // ==========================================================================
    // MOVE HANDLING
    // ==========================================================================

    function selectSquare(square) {
        if (state.animating || state.solved) return;
        
        const piece = state.pieces[square];
        
        if (!state.selectedSquare) {
            if (piece && isOwnPiece(square)) {
                state.selectedSquare = square;
                renderBoard();
            }
        } else {
            if (square === state.selectedSquare) {
                state.selectedSquare = null;
                renderBoard();
            } else if (piece && isOwnPiece(square)) {
                state.selectedSquare = square;
                renderBoard();
            } else {
                const from = state.selectedSquare;
                state.selectedSquare = null;
                attemptMove(from, square);
            }
        }
    }

    function isCastlingMove(from, to) {
        const piece = state.pieces[from];
        if (!piece || piece.toUpperCase() !== 'K') return false;
        const fromFile = FILES.indexOf(from[0]);
        const toFile = FILES.indexOf(to[0]);
        return Math.abs(toFile - fromFile) === 2;
    }

    function attemptMove(from, to) {
        if (state.animating) return;
        
        const move = from + to;
        const bestMove = state.puzzle.best_move.toLowerCase();
        const moveBase = move.toLowerCase();
        
        const isCorrect = bestMove === moveBase || 
                          (bestMove.startsWith(moveBase) && bestMove.length === 5);
        
        state.totalAttempts++;
        
        if (isCorrect) {
            const isCapture = !!state.pieces[to];
            const isCastle = isCastlingMove(from, to);
            state.animating = true;
            
            const afterMove = () => {
                state.lastMove = { from, to };
                state.animating = false;
                renderBoard();
                
                setTimeout(() => {
                    state.solved = true;
                    state.totalSolved++;
                    playSound('correct');
                    showFeedback('Correct!', true);
                    document.getElementById('btn-next').style.display = 'inline-block';
                    document.getElementById('btn-hint').style.display = 'none';
                    updateStats();
                    loadContinuation();
                }, 150);
            };
            
            if (isCastle) {
                animateCastling(from, to, () => {
                    executeCastling(from, to);
                    playSound('move');
                    afterMove();
                });
            } else {
                animateMove(from, to, () => {
                    state.pieces[to] = state.pieces[from];
                    delete state.pieces[from];
                    
                    if (bestMove.length === 5) {
                        const promoChar = bestMove[4];
                        const isWhite = state.turn === 'white';
                        state.pieces[to] = isWhite ? promoChar.toUpperCase() : promoChar.toLowerCase();
                    }
                    
                    playSound(isCapture ? 'capture' : 'move');
                    afterMove();
                });
            }
        } else {
            state.animating = true;
            animateWrongMove(from, to, () => {
                state.animating = false;
                renderBoard();
            });
            playSound('wrong');
            showFeedback('Try again', false);
            updateStats();
        }
    }
    
    function executeCastling(kingFrom, kingTo) {
        const rank = kingFrom[1];
        const isKingside = kingTo[0] === 'g';
        
        state.pieces[kingTo] = state.pieces[kingFrom];
        delete state.pieces[kingFrom];
        
        if (isKingside) {
            state.pieces['f' + rank] = state.pieces['h' + rank];
            delete state.pieces['h' + rank];
        } else {
            state.pieces['d' + rank] = state.pieces['a' + rank];
            delete state.pieces['a' + rank];
        }
        
        if (state.turn === 'white') {
            state.castling = state.castling.replace('K', '').replace('Q', '');
        } else {
            state.castling = state.castling.replace('k', '').replace('q', '');
        }
    }
    
    function executeMove(uciMove) {
        const from = uciMove.substring(0, 2);
        const to = uciMove.substring(2, 4);
        const promo = uciMove.length === 5 ? uciMove[4] : null;
        
        const isCapture = !!state.pieces[to];
        const isCastle = isCastlingMove(from, to);
        
        return new Promise(resolve => {
            const afterAnim = () => {
                state.turn = state.turn === 'white' ? 'black' : 'white';
                state.lastMove = { from, to };
                renderBoard();
                resolve();
            };
            
            if (isCastle) {
                animateCastling(from, to, () => {
                    executeCastling(from, to);
                    playSound('move');
                    afterAnim();
                });
            } else {
                animateMove(from, to, () => {
                    state.pieces[to] = state.pieces[from];
                    delete state.pieces[from];
                    
                    if (promo) {
                        const wasWhite = state.turn === 'white';
                        state.pieces[to] = wasWhite ? promo.toUpperCase() : promo.toLowerCase();
                    }
                    
                    playSound(isCapture ? 'capture' : 'move');
                    afterAnim();
                });
            }
        });
    }

    // ==========================================================================
    // ANIMATIONS
    // ==========================================================================

    function animateMove(from, to, callback) {
        const fromEl = getSquareElement(from);
        const toEl = getSquareElement(to);
        const pieceEl = fromEl?.querySelector('.piece');
        
        if (!fromEl || !toEl || !pieceEl) {
            callback();
            return;
        }
        
        const fromRect = fromEl.getBoundingClientRect();
        const toRect = toEl.getBoundingClientRect();
        
        const deltaX = toRect.left - fromRect.left;
        const deltaY = toRect.top - fromRect.top;
        
        pieceEl.style.transition = 'transform 0.12s cubic-bezier(0.25, 0.1, 0.25, 1)';
        pieceEl.style.transform = `translate(${deltaX}px, ${deltaY}px)`;
        pieceEl.style.zIndex = '100';
        
        setTimeout(() => {
            pieceEl.style.transition = '';
            pieceEl.style.transform = '';
            pieceEl.style.zIndex = '';
            callback();
        }, 120);
    }
    
    function animateCastling(kingFrom, kingTo, callback) {
        const rank = kingFrom[1];
        const isKingside = kingTo[0] === 'g';
        
        const rookFrom = isKingside ? ('h' + rank) : ('a' + rank);
        const rookTo = isKingside ? ('f' + rank) : ('d' + rank);
        
        const kingFromEl = getSquareElement(kingFrom);
        const kingToEl = getSquareElement(kingTo);
        const rookFromEl = getSquareElement(rookFrom);
        const rookToEl = getSquareElement(rookTo);
        
        const kingPiece = kingFromEl?.querySelector('.piece');
        const rookPiece = rookFromEl?.querySelector('.piece');
        
        if (!kingPiece || !rookPiece) {
            callback();
            return;
        }
        
        const kingFromRect = kingFromEl.getBoundingClientRect();
        const kingToRect = kingToEl.getBoundingClientRect();
        const rookFromRect = rookFromEl.getBoundingClientRect();
        const rookToRect = rookToEl.getBoundingClientRect();
        
        kingPiece.style.transition = 'transform 0.15s cubic-bezier(0.25, 0.1, 0.25, 1)';
        kingPiece.style.transform = `translate(${kingToRect.left - kingFromRect.left}px, ${kingToRect.top - kingFromRect.top}px)`;
        kingPiece.style.zIndex = '100';
        
        rookPiece.style.transition = 'transform 0.15s cubic-bezier(0.25, 0.1, 0.25, 1)';
        rookPiece.style.transform = `translate(${rookToRect.left - rookFromRect.left}px, ${rookToRect.top - rookFromRect.top}px)`;
        rookPiece.style.zIndex = '99';
        
        setTimeout(() => {
            kingPiece.style.transition = '';
            kingPiece.style.transform = '';
            kingPiece.style.zIndex = '';
            rookPiece.style.transition = '';
            rookPiece.style.transform = '';
            rookPiece.style.zIndex = '';
            callback();
        }, 150);
    }

    function animateWrongMove(from, to, callback) {
        const fromEl = getSquareElement(from);
        const toEl = getSquareElement(to);
        const pieceEl = fromEl?.querySelector('.piece');
        
        if (!fromEl || !toEl || !pieceEl) {
            callback();
            return;
        }
        
        const fromRect = fromEl.getBoundingClientRect();
        const toRect = toEl.getBoundingClientRect();
        
        const deltaX = toRect.left - fromRect.left;
        const deltaY = toRect.top - fromRect.top;
        
        pieceEl.style.transition = 'transform 0.06s ease-out';
        pieceEl.style.transform = `translate(${deltaX}px, ${deltaY}px)`;
        pieceEl.style.zIndex = '100';
        
        setTimeout(() => {
            pieceEl.style.transition = 'transform 0.12s cubic-bezier(0.175, 0.885, 0.32, 1.275)';
            pieceEl.style.transform = 'translate(0, 0)';
            
            setTimeout(() => {
                pieceEl.style.transition = '';
                pieceEl.style.zIndex = '';
                callback();
            }, 120);
        }, 60);
    }

    // ==========================================================================
    // CONTINUATION
    // ==========================================================================

    async function loadContinuation() {
        if (!state.puzzle) return;
        
        try {
            const response = await fetch(`/api/puzzles/${state.puzzle.id}/continuation`);
            if (!response.ok) return;
            
            const data = await response.json();
            if (data.moves && data.moves.length > 0) {
                showFeedback('Correct! Watch the continuation...', true);
                await playContinuation(data.moves);
                showFeedback('Correct! ' + formatLine(data.moves), true);
            }
        } catch (err) {
            console.log('Failed to load continuation:', err);
        }
    }
    
    async function playContinuation(moves) {
        state.animating = true;
        for (const move of moves) {
            await sleep(400);
            await executeMove(move);
        }
        state.animating = false;
    }
    
    function sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
    
    function formatLine(moves) {
        return moves.map(m => m.substring(0, 2) + '-' + m.substring(2, 4)).join(' ');
    }

    // ==========================================================================
    // DRAG AND DROP
    // ==========================================================================

    function initDragDrop() {
        const board = document.getElementById('chess-board');
        if (!board) return;
        
        board.addEventListener('mousedown', onMouseDown);
        document.addEventListener('mousemove', onMouseMove);
        document.addEventListener('mouseup', onMouseUp);
        
        board.addEventListener('touchstart', onTouchStart, { passive: false });
        document.addEventListener('touchmove', onTouchMove, { passive: false });
        document.addEventListener('touchend', onTouchEnd);
    }

    function createDragElement(pieceEl, x, y) {
        const rect = pieceEl.getBoundingClientRect();
        const clone = pieceEl.cloneNode(true);
        
        clone.style.cssText = `
            position: fixed;
            width: ${rect.width}px;
            height: ${rect.height}px;
            left: ${x - rect.width/2}px;
            top: ${y - rect.height/2}px;
            pointer-events: none;
            z-index: 1000;
            transform: scale(1.15);
            filter: drop-shadow(0 8px 16px rgba(0,0,0,0.4));
            cursor: grabbing;
        `;
        
        document.body.appendChild(clone);
        return clone;
    }

    function onMouseDown(e) {
        if (state.animating || state.solved) return;
        
        const squareEl = e.target.closest('.chess-square');
        const square = squareEl?.dataset.square;
        if (!square) return;
        
        if (!isOwnPiece(square)) return;
        
        e.preventDefault();
        
        state.dragging = square;
        state.dragStartX = e.clientX;
        state.dragStartY = e.clientY;
        state.hasDragged = false;
    }

    function onMouseMove(e) {
        if (!state.dragging) return;
        
        const dx = e.clientX - state.dragStartX;
        const dy = e.clientY - state.dragStartY;
        
        if (!state.hasDragged && Math.sqrt(dx*dx + dy*dy) > DRAG_THRESHOLD) {
            state.hasDragged = true;
            state.selectedSquare = state.dragging;
            
            const pieceEl = getSquareElement(state.dragging)?.querySelector('.piece');
            if (pieceEl) {
                pieceEl.style.opacity = '0.4';
                state.dragElement = createDragElement(pieceEl, e.clientX, e.clientY);
            }
            
            renderBoard();
            
            // Re-ghost the piece after render
            const newPieceEl = getSquareElement(state.dragging)?.querySelector('.piece');
            if (newPieceEl) newPieceEl.style.opacity = '0.4';
        }
        
        if (state.hasDragged && state.dragElement) {
            const size = state.dragElement.offsetWidth;
            state.dragElement.style.left = (e.clientX - size/2) + 'px';
            state.dragElement.style.top = (e.clientY - size/2) + 'px';
            
            // Hover highlight
            const hoverSquare = getSquareFromPoint(e.clientX, e.clientY);
            if (hoverSquare !== state.currentHoverSquare) {
                if (state.currentHoverSquare) {
                    const oldEl = getSquareElement(state.currentHoverSquare);
                    if (oldEl) oldEl.classList.remove('drag-over', 'can-drop');
                }
                
                if (hoverSquare && hoverSquare !== state.dragging) {
                    const newEl = getSquareElement(hoverSquare);
                    if (newEl) {
                        newEl.classList.add('drag-over');
                        const validMoves = getPseudoLegalMoves(state.dragging);
                        if (validMoves.includes(hoverSquare)) {
                            newEl.classList.add('can-drop');
                        }
                    }
                }
                
                state.currentHoverSquare = hoverSquare;
            }
        }
    }

    function onMouseUp(e) {
        // Clean up hover
        if (state.currentHoverSquare) {
            const hoverEl = getSquareElement(state.currentHoverSquare);
            if (hoverEl) hoverEl.classList.remove('drag-over', 'can-drop');
            state.currentHoverSquare = null;
        }
        
        // Clean up drag element
        if (state.dragElement) {
            state.dragElement.remove();
            state.dragElement = null;
        }
        
        // Restore piece opacity
        if (state.dragging) {
            const pieceEl = getSquareElement(state.dragging)?.querySelector('.piece');
            if (pieceEl) pieceEl.style.opacity = '';
        }
        
        const wasDragging = state.dragging;
        const hadDragged = state.hasDragged;
        const dropSquare = getSquareFromPoint(e.clientX, e.clientY);
        
        state.dragging = null;
        state.hasDragged = false;
        
        if (hadDragged && wasDragging && dropSquare && dropSquare !== wasDragging) {
            state.selectedSquare = null;
            attemptMove(wasDragging, dropSquare);
        } else if (!hadDragged && wasDragging) {
            selectSquare(wasDragging);
        } else if (!wasDragging && dropSquare) {
            selectSquare(dropSquare);
        } else {
            state.selectedSquare = null;
            renderBoard();
        }
    }

    function getSquareFromPoint(x, y) {
        if (state.dragElement) state.dragElement.style.display = 'none';
        const elements = document.elementsFromPoint(x, y);
        if (state.dragElement) state.dragElement.style.display = '';
        
        for (const el of elements) {
            if (el.classList.contains('chess-square')) {
                return el.dataset.square;
            }
        }
        return null;
    }

    function onTouchStart(e) {
        const touch = e.touches[0];
        onMouseDown({ 
            target: document.elementFromPoint(touch.clientX, touch.clientY),
            clientX: touch.clientX, 
            clientY: touch.clientY,
            preventDefault: () => e.preventDefault()
        });
    }

    function onTouchMove(e) {
        if (!state.dragging) return;
        e.preventDefault();
        const touch = e.touches[0];
        onMouseMove({ clientX: touch.clientX, clientY: touch.clientY });
    }

    function onTouchEnd(e) {
        const touch = e.changedTouches[0];
        onMouseUp({ clientX: touch.clientX, clientY: touch.clientY });
    }

    // ==========================================================================
    // UI
    // ==========================================================================

    function showFeedback(message, success) {
        const feedback = document.getElementById('feedback');
        if (feedback) {
            feedback.textContent = message;
            feedback.className = 'feedback ' + (success ? 'correct' : 'wrong');
        }
    }

    function updateStats() {
        document.getElementById('stat-solved').textContent = state.totalSolved;
        document.getElementById('stat-attempts').textContent = state.totalAttempts;
        
        const accuracy = state.totalAttempts > 0 
            ? Math.round((state.totalSolved / state.totalAttempts) * 100)
            : 0;
        document.getElementById('stat-accuracy').textContent = accuracy + '%';
    }

    function updatePuzzleInfo() {
        if (!state.puzzle) return;
        
        const severity = document.getElementById('info-severity');
        const cpLoss = document.getElementById('info-cp-loss');
        
        if (severity) {
            const severityColors = {
                'blunder': '#e64c4c',
                'mistake': '#e6a54c',
                'inaccuracy': '#c9c94c'
            };
            const color = severityColors[state.puzzle.severity] || 'inherit';
            severity.innerHTML = `<strong>Severity:</strong> <span style="color: ${color}">${state.puzzle.severity}</span>`;
        }
        if (cpLoss && state.puzzle.cp_loss) {
            cpLoss.innerHTML = `<strong>CP Loss:</strong> ${state.puzzle.cp_loss}`;
        }
        
        const context = document.getElementById('puzzle-context');
        if (context) {
            context.innerHTML = `<span style="color: var(--highlight-wrong);">You played ${state.puzzle.player_move}</span> — Find the better move!`;
        }
    }

    // ==========================================================================
    // API
    // ==========================================================================

    async function loadPuzzle() {
        try {
            const response = await fetch('/api/puzzles/next');
            if (!response.ok) {
                showFeedback('No puzzles available', false);
                return;
            }
            
            state.puzzle = await response.json();
            const parsed = parseFen(state.puzzle.fen);
            state.pieces = parsed.pieces;
            state.turn = parsed.turn;
            state.castling = parsed.castling;
            state.perspective = parsed.turn;
            state.solved = false;
            state.selectedSquare = null;
            state.dragging = null;
            state.hasDragged = false;
            state.lastMove = null;
            state.animating = false;
            state.currentHoverSquare = null;
            
            renderBoard();
            updatePuzzleInfo();
            
            document.getElementById('feedback').textContent = '';
            document.getElementById('feedback').className = 'feedback';
            document.getElementById('btn-next').style.display = 'none';
            document.getElementById('btn-hint').style.display = 'inline-block';
            
        } catch (err) {
            console.error('Failed to load puzzle:', err);
            showFeedback('Failed to load puzzle', false);
        }
    }

    function showHint() {
        if (!state.puzzle) return;
        
        const bestMove = state.puzzle.best_move;
        const from = bestMove.substring(0, 2);
        
        document.querySelectorAll('.hint-from').forEach(el => el.classList.remove('hint-from'));
        
        const el = getSquareElement(from);
        if (el) el.classList.add('hint-from');
        
        showFeedback(`Hint: Move the piece on ${from}`, false);
    }

    // ==========================================================================
    // INIT
    // ==========================================================================

    function init() {
        initDragDrop();
        loadPuzzle();
        
        document.getElementById('btn-next')?.addEventListener('click', loadPuzzle);
        document.getElementById('btn-hint')?.addEventListener('click', showHint);
        document.getElementById('btn-reset')?.addEventListener('click', () => {
            state.totalSolved = 0;
            state.totalAttempts = 0;
            updateStats();
            loadPuzzle();
        });
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

})();
