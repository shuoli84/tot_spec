# import annotations to enable forward declaration
from __future__ import annotations
from dataclasses import dataclass
import abc
import typing
import decimal

from . import include_base as base
from . import include_base as base_dup

@dataclass
class TestBase:
    # use base's BaseId as the id
    id: base.Id
    # use base_dup's BaseId as the id_2, this is just demo
    id_2: base_dup.Id
    common: base.Common

    def to_dict(self):
        result = {}

        # id
        id_tmp = self.id
        result["id"] = id_tmp

        # id_2
        id_2_tmp = self.id_2
        result["id_2"] = id_2_tmp

        # common
        common_tmp = self.common.to_dict()
        result["common"] = common_tmp
        return result


    @staticmethod
    def from_dict(d):

        # id
        id = int(d["id"])

        # id_2
        id_2 = int(d["id_2"])

        # common
        common = base.Common.from_dict(d["common"])
        return TestBase(
            id = id,
            id_2 = id_2,
            common = common,
        )


