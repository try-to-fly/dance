import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { ShortcutRecorder } from './ShortcutRecorder';

describe('ShortcutRecorder', () => {
  it('使用物理按键码记录 option 修饰后的字母键', async () => {
    const onChange = vi.fn();
    const onValidate = vi.fn().mockResolvedValue(true);
    const user = userEvent.setup();

    render(
      <ShortcutRecorder value="CmdOrCtrl+Shift+V" onChange={onChange} onValidate={onValidate} />
    );

    await user.click(screen.getByRole('button', { name: '更改' }));

    fireEvent.keyDown(document, {
      key: 'ç',
      code: 'KeyC',
      metaKey: true,
      altKey: true,
    });

    await user.click(screen.getByTitle('确认'));

    await waitFor(() => {
      expect(onValidate).toHaveBeenCalledWith('CmdOrCtrl+Alt+C');
      expect(onChange).toHaveBeenCalledWith('CmdOrCtrl+Alt+C');
    });
  });

  it('不支持的主键会显示错误', async () => {
    const user = userEvent.setup();

    render(<ShortcutRecorder value="" onChange={vi.fn()} />);

    await user.click(screen.getByRole('button', { name: '更改' }));

    fireEvent.keyDown(document, {
      key: '§',
      code: 'IntlBackslash',
      altKey: true,
      metaKey: true,
    });

    expect(screen.getByText('该按键暂不支持作为快捷键主键，请换一个键')).toBeInTheDocument();
  });
});
