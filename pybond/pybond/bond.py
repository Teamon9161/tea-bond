import os
from datetime import date
from importlib.util import find_spec
from pathlib import Path

from .pybond import Bond as _BondRS
from .pybond import download_bond

WIND_AVAILABLE = find_spec("WindPy") is not None
if WIND_AVAILABLE:
    from .download import login as wind_login

if os.environ.get("BONDS_INFO_PATH") is not None:
    bonds_info_environ_flag = True
    bonds_info_path = Path(os.environ.get("BONDS_INFO_PATH"))
else:
    bonds_info_environ_flag = False
    bonds_info_path = Path(__file__).parent / "data" / "bonds_info"

if not bonds_info_path.exists():
    bonds_info_path.mkdir(parents=True)


class Bond(_BondRS):
    def __new__(cls, code: str | int, path: str | Path | None = None):
        """
        Create a new Bond instance.

        Args:
            code (str | int): The bond code. If no extension is provided, '.IB' will be appended.
            path (str | Path | None): Path to the bond info file. If None, uses default bonds_info_path.

        Returns:
            Bond: A new Bond instance, either loaded from existing JSON file or downloaded.

        Note:
            If a JSON file for the bond code doesn't exist at the specified path,
            the bond info will be downloaded automatically.
        """
        code = str(code)
        if "." not in code:
            code = code + ".IB"
        path = bonds_info_path if path is None else Path(path)
        if (path / (code + ".json")).exists():
            return super().__new__(cls, code, path)
        else:
            cls.download(code, path)
            return super().__new__(cls, code, path)

    @staticmethod
    def download(
        code: str, path: str | None = None, source: str | None = None, save=True
    ):
        """
        Download bond information from a specified source.

        This method downloads bond information for a given bond code from either Wind or Rust.
        If no source is specified, it defaults to Wind if the WindPy module is available; otherwise,
        it falls back to Rust.

        If the source is 'rust', the method will download IB bond information from China Money and
        SH bond information from SSE (Shanghai Stock Exchange).

        Args:
            code (str): The bond code in the format 'XXXXXX.YY'. The code must include a dot.
            path (str | None): The directory path where the downloaded bond information should be saved.
                              If None, the default path is used.
            source (str | None): The source from which to download the bond information. Valid options are
                                'wind' or 'rust'. If None, the source is automatically determined.
            save (bool): Whether to save the downloaded bond information to the specified path.
                        Defaults to True.

        Returns:
            Bond: The downloaded bond object if the source is 'rust' and save is False.
                  Otherwise, returns None.

        Raises:
            AssertionError: If the code is not in the correct format or if the source is invalid.
        """
        if source is None:
            # 优先从wind下载
            source = "wind" if WIND_AVAILABLE else "rust"
        assert "." in code, "code should be in the format of XXXXXX.YY"
        assert source in ("wind", "rust")
        if source == "wind":
            from .download import fetch_symbols, login

            print(f"Start downloading bond info for {code} from Wind")
            login()
            fetch_symbols([code], save=save, save_folder=path)
        else:
            # let rust side handle the download
            print(f"download {code}")
            bond = download_bond(code)
            if save:
                bond.save(path)
            return bond

    def accrued_interest(
        self, date: date, cp_dates: tuple[date, date] | None = None
    ) -> float:
        """
        计算应计利息

        银行间和交易所的计算规则不同,银行间是算头不算尾,而交易所是算头又算尾
        """
        return self.calc_accrued_interest(date, cp_dates=cp_dates)

    def dirty_price(
        self,
        ytm: float,
        date: date,
        cp_dates: tuple[date, date] | None = None,
        remain_cp_num: int | None = None,
    ) -> float:
        """通过ytm计算债券全价"""
        return self.calc_dirty_price_with_ytm(
            ytm, date, cp_dates=cp_dates, remain_cp_num=remain_cp_num
        )

    def clean_price(
        self,
        ytm: float,
        date: date,
        cp_dates: tuple[date, date] | None = None,
        remain_cp_num: int | None = None,
    ) -> float:
        """通过ytm计算债券净价"""
        return self.calc_clean_price_with_ytm(
            ytm, date, cp_dates=cp_dates, remain_cp_num=remain_cp_num
        )

    def macaulay_duration(
        self,
        ytm: float,
        date: date,
        cp_dates: tuple[date, date] | None = None,
        remain_cp_num: int | None = None,
    ) -> float:
        """计算麦考利久期"""
        return self.calc_macaulay_duration(
            ytm, date, cp_dates=cp_dates, remain_cp_num=remain_cp_num
        )

    def duration(
        self,
        ytm: float,
        date: date,
        cp_dates: tuple[date, date] | None = None,
        remain_cp_num: int | None = None,
    ) -> float:
        """计算修正久期"""
        return self.calc_duration(
            ytm, date, cp_dates=cp_dates, remain_cp_num=remain_cp_num
        )
